# Task 97 Packet 004: qjl32 Local AVX2 Kernel

This packet adds the locally verifiable AVX2 qjl32 kernel slice for the
clarified Task 97 path: QJL-active canonical `bits=4` TurboQuant at non-tiled
dimensions such as 1024.

## Changes

- Implements `src/quant/qjl32/avx2.rs`.
- Keeps runtime AVX2 feature detection and returns `Isa::Avx2` only when the
  AVX2 path is used; otherwise it falls back to scalar and returns
  `Isa::Scalar`.
- Adds a focused AVX2 tolerance test against the scalar reference.
- Adds `bench_api` qjl32 hooks and a filtered Criterion row in
  `benches/criterion/quant_score.rs` for scalar-vs-dispatch local evidence.

## Validation

- `cargo test qjl32 --lib -- --color never`
  - 5 passed; 0 failed
- `cargo test candidate_batch --lib -- --color never`
  - 17 passed; 0 failed
- `cargo bench --features bench --bench quant_score qjl32 -- --sample-size 10 --warm-up-time 1 --measurement-time 2`
  - scalar median: 36.629 us per 32 candidates
  - dispatch median: 28.355 us per 32 candidates
  - local median speedup: 1.29x

Logs are under `artifacts/`.

## Review Request

Please review the AVX2 qjl32 implementation, the tolerance test, and the local
Criterion evidence. This packet does not include NEON/SVE2 or AWS evidence.
