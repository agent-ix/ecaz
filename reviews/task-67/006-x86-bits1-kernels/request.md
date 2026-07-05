# Task 67: x86 bits=1 RaBitQ Kernels

## Summary

This packet lands x86 single-candidate bits=1 RaBitQ scoring:

- `Avx512Bits1` now dispatches to `sum_query_dequant_avx512_bits1` behind AVX-512F+VPOPCNTDQ runtime selection.
- `Avx2Bits1` now dispatches to `sum_query_dequant_avx2_bits1` behind AVX2+FMA runtime selection.
- Both kernels preserve the existing weighted bits=1 estimator by multiplying query lanes against the prepared per-query byte LUT.
- Added a runtime-gated scalar differential test for AVX-512F+VPOPCNTDQ and AVX2+FMA bits=1 kernels.

Code commit: `664376fd478b0d7fc5dd813f314c1a6561f09732`

## Scope Notes

The task text describes the bits=1 fast path as a sign-popcount inner product. The production code path is query-weighted through `bits1_byte_lut`, so this slice keeps that semantic contract instead of substituting an unweighted Hamming/popcount score.

This advances single-candidate bits=1 coverage. It does not claim batched bits=1, bf16, Intel runtime execution, benchmark, recall, or Slice J measurement gates.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits1_matches_scalar_when_available --no-run` completed successfully.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile evidence only.

## Review Focus

- Confirm the x86 bits=1 kernels preserve byte order and bit order relative to the scalar byte-LUT path.
- Confirm `Avx512Bits1` and `Avx2Bits1` are no longer downgraded to scalar after slot selection.
- Confirm the AVX-512 feature gate is acceptable even though the correctness-preserving implementation uses the weighted byte-LUT path rather than a plain popcount reduction.
