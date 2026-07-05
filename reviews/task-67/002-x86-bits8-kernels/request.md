# Task 67: x86 bits=8 RaBitQ Kernels

## Summary

This packet lands x86 single-candidate arithmetic-dequant scoring for the active RaBitQ bits=8 path:

- `Avx512Bits8` now dispatches to `sum_query_dequant_avx512_bits8` behind AVX-512F runtime detection.
- `Avx2Bits8` now dispatches to `sum_query_dequant_avx2_bits8` behind AVX2+FMA runtime detection.
- Both kernels use the Task 66 bits=8 precompute shape: `sum += code_byte * query_scale + query_offset`.
- The reserved x86 bits=1, bits=4, and bf16 slots still fall back to scalar.
- Added a runtime-gated scalar differential test for AVX-512F and AVX2+FMA bits=8 kernels.

Code commit: `434a2da562e7e3547b3e154d97a80ad2e98ba845`

## Scope Notes

This advances Task 67's bits=8 coverage for `rabitq8`, `rabitq8ls`, `rabitq8c3`, and `rabitq8c4`. It does not claim Slice J measurement gates or batched scoring yet.

The planned Slice B bits=1 VPOPCNTDQ work needs care: the current production bits=1 scorer is query-weighted through `query_rotated` and a byte dequant LUT, not a pure Hamming/popcount score. I left that slot scalar rather than adding a mathematically mismatched popcount kernel.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits8_matches_scalar_when_available --no-run` completed successfully.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile evidence only.

## Review Focus

- Confirm the AVX-512F and AVX2+FMA bits=8 kernels preserve the scalar arithmetic-dequant formula.
- Confirm the safety comments and runtime feature gates are sufficient.
- Confirm the decision to leave bits=1 scalar until the query-weighted math is reconciled with the task's VPOPCNTDQ wording.
