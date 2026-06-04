# Review Request: RaBitQ Radius-Adjusted Block Pruning

Code commit under review: `2a7c7a089ffe5e45344c32001c9139c0e6cd0c55`

This checkpoint responds directly to the packet 011 result: mean-only leaf block
pruning reduced candidates and latency but destroyed recall. The selector now
uses a radius-adjusted score for RaBitQ block summaries instead of ranking only
by the encoded block centroid.

## What Changed

- Added `query_l2_norm` to `SpirePreparedAssignmentScorer`.
- During RaBitQ leaf block summary build, `summary.gamma` now stores the max L2
  residual radius from the block mean to any source vector in the block.
- During RaBitQ summary selection, the scanner scores each block as
  `encoded_mean_ip + query_l2_norm * radius`.
- TurboQuant summary semantics are left unchanged: non-RaBitQ summaries still
  preserve their encoded-payload gamma.
- Added focused tests for radius materialization and selector ranking.

## Why This Slice

Packet 011 showed the first direct pruning mechanism worked mechanically:
block64/prune4 cut the candidate surface from `15,506,227` to `4,547,347` and
p50 from `62.907 ms` to `37.292 ms`, but recall fell from `0.9975` to `0.7790`.
That says the failure is block choice, not candidate-budget enforcement.

The new score is a conservative next step: a block with a mediocre centroid can
still survive pruning if it has enough radius to contain a high-scoring vector.
It keeps the storage shape stable by using the RaBitQ-only summary gamma slot,
which row scoring already requires to be zero but the summary selector can
interpret separately.

## Validation

See `artifacts/manifest.md`.

- `cargo fmt --check`: pass with existing nightly-only rustfmt config warnings
- `cargo check -p ecaz`: pass
- `cargo test -p ecaz leaf_block_summaries_cover_rabitq_row_blocks`: pass
- `cargo test -p ecaz select_leaf_block_row_ranges`: pass, including the new
  radius-ranking test

## Next Required Evidence

Run `ecaz bench suite` on the primary RaBitQ 100k nprobe96 lane after rebuilding
the index with the radius summaries. The acceptance question is whether the
radius-adjusted selector keeps the packet 011 candidate/p50 gains while moving
recall back into the Task 79 target band.
