# Task 66 M5 NEON RaBitQ Optimization

## Summary

Implements the full Task 66 first pass for RaBitQ scoring on Apple Silicon while preserving scalar/x86 fallback behavior and leaving an explicit dispatch slot for Task 67 Intel work.

Code checkpoint: `91f2e51ec9e7825f4f6710eac11bbe75408281fe`.

## What Changed

- Added a cross-arch RaBitQ scoring dispatch seam in `src/quant/rabitq.rs`.
- Added bits=8 query-side arithmetic-dequant precompute and a NEON bits=8 kernel.
- Replaced the fake bits=1-only batch path with a bits=1/bits=8 batch scorer.
- Added `prfm pldl1keep` prefetch hints to bits=1 and bits=8 NEON kernels.
- Extended IVF scratch-SoA batch scoring to RaBitQ `quant_bits=8`.
- Updated sidecar-rerank to batch-score `rabitq8`, `rabitq8c3`, and `rabitq8c4` slabs.
- Added Criterion coverage for RaBitQ bits=1/4/8 scoring and batch scoring.
- Re-measured the `rabitq-bf16` path on M5 and left it off by default because Criterion detected no performance win.

## Validation

- `cargo check --no-default-features --features pg18` passed.
- `cargo test --lib --no-default-features --features pg18 quant::rabitq` passed: 41 tests.
- `cargo check -p ecaz-cli` passed.
- `cargo check --benches --features bench --no-default-features --features pg18` passed.
- M5 Criterion logs are in `artifacts/criterion-rabitq-neon.log` and `artifacts/criterion-rabitq-bf16.log`.

## Key M5 Results

- bits=8 single-score: `123.71-127.05 ns`.
- bits=8c3 single-score: `123.07-125.31 ns`.
- bits=8c4 single-score: `121.25-122.37 ns`.
- bits=4 f32 NEON: `233.24-240.52 ns`.
- bits=4 with `rabitq-bf16`: `233.02-239.31 ns`; Criterion reported no significant change.

## Review Notes

- The x86 dispatch arm intentionally falls back to scalar today; Task 67 can add AVX2/AVX-512 variants at the `QueryDequantKernel` seam.
- The bf16 feature remains opt-in.
- Recall/SQL suite evidence is not included in this first packet because these changes are scoring-kernel equivalent and covered by scalar/NEON differential tests; follow-up full IVF sidecar suites can reuse the new Criterion group and sidecar-rerank batch path.
