---
agent: coder-1
role: coder
model: gpt-5
date: 2026-06-09
seq: 05
---

# Task 94 Phase 5 Review Request — AVX2 Grouped-PQ Block Backend

## Scope

This packet implements the Phase 5 AVX2 backend for the grouped-PQ /
PqFastScan 32-candidate block kernel.

Code checkpoint:

- `1ebe3652b Add grouped-PQ AVX2 block backend`

## What Changed

- `src/quant/grouped_pq_block/avx2.rs` now has a real AVX2 backend.
- The backend processes eight candidates per vector and gathers from the
  canonical row-major f32 LUT with `_mm256_i32gather_ps`.
- Runtime AVX2 detection gates the backend; unsupported hosts still delegate to
  scalar and return `Isa::Scalar`.
- Added a dedicated AVX2 parity test that requires `Isa::Avx2` when AVX2 is
  available.
- Loosened the general block32 test's ISA assertion so real SIMD backends do
  not make it fail.

## Validation

Local only:

```text
cargo test grouped_pq_block --lib
```

Result: 8 passed, 0 failed. See `artifacts/test-grouped-pq-block.log`.

No CI, AWS, or benchmark runs were performed.

