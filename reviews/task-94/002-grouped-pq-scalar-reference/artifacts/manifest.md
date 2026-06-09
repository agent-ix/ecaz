# Task 94 Phase 2 Manifest

- head SHA: `360429525cca9201ce8582f945dd85481b0d33de`
- task bucket: `reviews/task-94/`
- packet path: `reviews/task-94/002-grouped-pq-scalar-reference/`
- phase: Phase 2 scalar reference + bit-exact parity tests
- lane: LUT kernel family / grouped-PQ PqFastScan
- timestamp: `2026-06-09T16:50:45Z`
- code checkpoint: `360429525 Add grouped-PQ scalar block kernel`

## Changes

- Added `src/quant/grouped_pq_block/` with ADR-076 module layout:
  `mod.rs`, `scalar.rs`, `neon.rs`, `sve.rs`, `avx2.rs`.
- Added 32-candidate scalar block scorer:
  `score_grouped_pq_block32(lut, group_count, codes, out_scores) -> Isa`.
- Added scalar tail scorer:
  `score_grouped_pq_scalar(lut, group_count, code)`.
- Added shape validation helpers for LUT length and packed grouped-PQ code
  length.
- Added ISA fallback stubs for NEON, SVE, and AVX2 that delegate to scalar and
  return `Isa::Scalar`.
- Exported the module from `src/quant/mod.rs`.

## Validation

Artifact:

- `test-grouped-pq-block.log`

Command:

```text
cargo test grouped_pq_block --lib
```

Result:

```text
running 5 tests
test quant::grouped_pq_block::tests::grouped_pq_batch_rejects_shape_mismatch ... ok
test quant::grouped_pq_block::tests::grouped_pq_batch_under_block_width_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_batch_with_block_and_tail_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_scalar_tail_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_block32_matches_scalar_reference_bits_across_shapes ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2039 filtered out; finished in 0.00s
```

No CI, AWS, or benchmark runs were performed.

