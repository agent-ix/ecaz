# Review Request: A4 1536 Tiled-FWHT Quantizer Path

Basis: `main` at `2d14bea` plus the current working tree

## Summary

- implement a minimal production tiled-FWHT path for the `1536` compatibility tier
- keep the existing storage and score contract intact:
  - same payload layout
  - same `gamma` / MSE / QJL fields
  - same codebook parameterization (`lloyd_max(..., dim)`)
- limit the change to the quantizer path:
  - `src/quant/hadamard.rs`
  - `src/quant/rotation.rs`
  - `src/quant/prod.rs`
- use block-diagonal `3 x 512` orthonormal FWHT for `1536` instead of padding to `2048`

## Why This Slice Exists

Packet `202` showed that the dominant exact-only loss at `1536` was transform-tail truncation, not
graph traversal and not a codebook-only mismatch. The best next move was to make the production
quantizer path follow that evidence directly.

## Implementation

### 1. Tiled Hadamard helpers

- add `fwht_tiled_in_place(values, tile_size)`
- add `orthonormal_fwht_tiled_in_place(values, tile_size)`

These operate over equal-sized power-of-two chunks and keep the code path scalar and simple for
this lane.

### 2. Rotation policy for `1536`

- keep `transform_dim(dim)` unchanged as the padded power-of-two helper
- add `tile_dim(dim)` and `effective_transform_dim(dim)`
- choose:
  - `1536 -> effective_transform_dim = 1536`, `tile_dim = 512`
  - all other dimensions keep the padded full-SRHT path

`srht()` / `inverse_srht()` now dispatch to tiled FWHT only for the `1536` compatibility tier.

### 3. Production quantizer switch

`ProdQuantizer::new()` now uses `rotation::effective_transform_dim(dim)` rather than always using
the padded `next_power_of_two(dim)` working length.

That means:

- `ProdQuantizer::new(1536, ...)` now works in tiled `1536` space
- `ProdQuantizer::new(1024, ...)`, `2048`, etc. keep the old path

## New Results

### Clustered `1k x 1536`, `4-bit`

After the production change:

- `current_exact`: Recall@10 `81.5%`
- `transform_cb_exact`: `79.5%`
- `tail_ref_cb1536`: `80.5%`
- `tail_ref_cb2048`: `80.5%`

Read:

- the production quantizer path now matches the tiled-reference quality on this structured corpus
- the main quality recovery is real in the live implementation, not just in an offline probe
- the codebook-only substitution remains secondary

### Uniform `1k x 1536`, `4-bit`

After the production change:

- `current_exact`: Recall@10 `77.0%`
- `transform_cb_exact`: `80.5%`
- `tail_ref_cb1536`: `82.0%`
- `tail_ref_cb2048`: `80.5%`

Read:

- tiled FWHT still produces a large recovery on the harder uniform corpus
- there is still some remaining headroom versus the best reference variant
- codebook parameterization is still mixed rather than dead:
  - secondary on clustered data
  - somewhat helpful on uniform data

## What Worked

- the production change moved the exact-only ceiling sharply upward without changing payload size
- the `1536` path no longer throws away the `2048` padded transform tail because it no longer pads
  into that space
- the new quantizer path is covered by cheap permanent guards:
  - tiled SRHT roundtrip in `rotation.rs`
  - `ProdQuantizer::new(1536, ...)` explicitly uses a `1536` working dimension
  - a new ignored pg-test probes the live `1k` graph-first path and passed with:
    - exact Recall@10 `>= 70%`
    - graph Recall@10 `>= 70%`

## What Did Not Work Or Remains Secondary

- codebook-only substitution is still not the main fix
  - it regressed on clustered `1k` after the production change
  - it helped on uniform `1k`, but the major jump already came from tiling
- this packet does **not** yet prove the graph-first path has recovered proportionally
- this packet does **not** yet clear A4

## Current Read

This is the first production-path evidence that the quantized ceiling can move materially upward at
`1536` without changing storage format.

That changes the priority order:

1. keep the tiled-FWHT quantizer path
2. keep using the new `1k` live graph probe as the cheap runtime checkpoint
3. only then decide whether the next blocker is still graph-runtime or whether the `10k` gate is
   ready for a rerun

## Review Focus

- whether the production tiled-FWHT change is scoped appropriately for the A4 lane
- whether keeping the current codebook parameterization (`dim`, not `tile_dim`) is the right
  minimal choice for this slice
- whether the next evidence step should be `1k` live graph/runtime before any broader A4 rerun
