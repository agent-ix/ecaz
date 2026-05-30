# Task 67: x86 bits=4 RaBitQ Kernels

## Summary

This packet lands x86 single-candidate bits=4 RaBitQ scoring:

- `Avx512Bits4` now dispatches to `sum_query_dequant_avx512_bits4` behind AVX-512F+BW runtime selection.
- `Avx2Bits4` now dispatches to `sum_query_dequant_avx2_bits4` behind AVX2+FMA runtime selection.
- Both kernels preserve the scalar nibble decode order and accumulate query/dequant products with SIMD FMA lanes.
- Added a runtime-gated scalar differential test for AVX-512F+BW and AVX2+FMA bits=4 kernels.

Code commit: `497d2890e277f8fbea7fe68e447c4b68e56f1489`

## Scope Notes

This advances Task 67's bits=4 coverage. It does not claim bits=1, bf16, batched bits=4, Intel runtime execution, benchmark, recall, or Slice J measurement gates.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits4_matches_scalar_when_available --no-run` completed successfully.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile evidence only.

## Review Focus

- Confirm the AVX-512F+BW and AVX2+FMA bits=4 kernels preserve scalar nibble order.
- Confirm the runtime feature gates match each target-feature function.
- Confirm the scalar tail covers non-multiple-of-16 dimensions correctly.
