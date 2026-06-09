# Task 94 Phase 3 Manifest

- head SHA: `16872ca0fe4041e7bfeab4a31d8842898ece0462`
- task bucket: `reviews/task-94/`
- packet path: `reviews/task-94/003-grouped-pq-neon-backend/`
- phase: Phase 3 NEON backend
- lane: LUT kernel family / grouped-PQ PqFastScan
- timestamp: `2026-06-09T17:05:51Z`
- code checkpoint: `16872ca0f Add grouped-PQ NEON block backend`

## Changes

- Replaced the grouped-PQ NEON fallback stub with an aarch64 NEON backend.
- The backend processes four candidates per vector lane group, keeps the group
  loop in scalar-reference order for each lane, and returns `Isa::Neon` when
  runtime NEON detection succeeds.
- Non-aarch64 and missing-NEON hosts still delegate to the scalar block scorer
  and return `Isa::Scalar`.
- Added a NEON-specific test hook that executes the real backend when NEON is
  available and otherwise returns `None`.
- Added a conditional unit test that compares the NEON backend to
  `grouped_pq_score_f32(...).to_bits()` when the local host has NEON.

## Validation

Artifact:

- `test-grouped-pq-block.log`

Command:

```text
cargo test grouped_pq_block --lib
```

Result:

```text
running 6 tests
test quant::grouped_pq_block::tests::grouped_pq_batch_rejects_shape_mismatch ... ok
test quant::grouped_pq_block::tests::grouped_pq_batch_with_block_and_tail_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_batch_under_block_width_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_neon_backend_matches_scalar_reference_bits_when_available ... ok
test quant::grouped_pq_block::tests::grouped_pq_block32_matches_scalar_reference_bits_across_shapes ... ok
test quant::grouped_pq_block::tests::grouped_pq_scalar_tail_matches_scalar_reference_bits ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2039 filtered out; finished in 0.00s
```

## Evidence Limits

This run was local only. No CI, AWS, or benchmark runs were performed.

The local test validates build/fallback behavior on this host and executes the
real NEON backend only when the host exposes NEON. Graviton-4 runtime dispatch
evidence was not collected in this packet because AWS testing is reserved for
approved/final evidence runs.

