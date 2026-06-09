# Task 94 Phase 5 Manifest

- head SHA: `1ebe3652b6222364cfed76eb2591dae19e60b859`
- task bucket: `reviews/task-94/`
- packet path: `reviews/task-94/005-grouped-pq-avx2-backend/`
- phase: Phase 5 AVX2 backend
- lane: LUT kernel family / grouped-PQ PqFastScan
- timestamp: `2026-06-09T17:18:56Z`
- code checkpoint: `1ebe3652b Add grouped-PQ AVX2 block backend`

## Changes

- Replaced the grouped-PQ AVX2 fallback stub with an x86/x86_64 AVX2 backend.
- The backend processes eight candidates per vector and uses
  `_mm256_i32gather_ps` against the canonical f32 LUT.
- Runtime AVX2 detection gates the backend; unsupported hosts still delegate to
  scalar and return `Isa::Scalar`.
- Added a dedicated AVX2 parity test hook.
- Relaxed the general block parity test's ISA assertion so it remains valid on
  hosts with real SIMD backends.

## Validation

Artifact:

- `test-grouped-pq-block.log`

Command:

```text
cargo test grouped_pq_block --lib
```

Result:

```text
running 8 tests
test quant::grouped_pq_block::tests::grouped_pq_batch_rejects_shape_mismatch ... ok
test quant::grouped_pq_block::tests::grouped_pq_avx2_backend_matches_scalar_reference_bits_when_available ... ok
test quant::grouped_pq_block::tests::grouped_pq_batch_with_block_and_tail_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_neon_backend_matches_scalar_reference_bits_when_available ... ok
test quant::grouped_pq_block::tests::grouped_pq_batch_under_block_width_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_scalar_tail_matches_scalar_reference_bits ... ok
test quant::grouped_pq_block::tests::grouped_pq_block32_matches_scalar_reference_bits_across_shapes ... ok
test quant::grouped_pq_block::tests::grouped_pq_sve_backend_matches_scalar_reference_bits_when_available ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2039 filtered out; finished in 0.00s
```

No CI, AWS, or benchmark runs were performed.

