# Review Request: C1 Native Build Seed Copy Cleanup

Current head at execution: `a344ad9`

## Context

After the query-side and backlink-side score-cache slices, one remaining tiny
allocation in the native serial builder was still gratuitous: upper-layer seed
vectors were cloned before writing forward slots even though
`BeamCandidate<usize>` is `Copy`.

This is a very small checkpoint, but it is behavior-preserving and keeps the
serial native builder incrementally cheaper without widening scope.

## What changed

In `src/am/build.rs`:

- replaced `seeds.clone()` with `seeds.iter().copied()` when writing upper-layer
  forward candidates inside `populate_native_upper_layer_forward_slots(...)`

The builder still:

- keeps the same `seeds` vector for carrydown into the next layer
- writes the same forward slots in the same order
- preserves the same final seed set returned to layer 0

## Why this is safe

- `BeamCandidate<usize>` is `Copy`
- iteration order is unchanged
- no search, ranking, or tie-break logic changed
- no persisted layout or SQL surface changed

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Validation ran sequentially for this checkpoint.

## Review focus

1. Is it still worth carrying these tiny behavior-preserving cleanups as narrow
   checkpoints, or do you want the branch to stop optimization slices here and
   focus only on final landing evidence?
