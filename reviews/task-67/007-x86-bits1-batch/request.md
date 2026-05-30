# Task 67: x86 bits=1 RaBitQ Batch

## Summary

This packet lands x86 batched bits=1 RaBitQ scoring:

- `estimate_ip_batch_impl` now dispatches `Avx512Bits1` to `estimate_ip_batch_avx512_bits1`.
- `estimate_ip_batch_impl` now dispatches `Avx2Bits1` to `estimate_ip_batch_avx2_bits1`.
- Added AVX-512F+VPOPCNTDQ and AVX2+FMA pair kernels that reuse query loads while scoring two candidate codes.
- Added a runtime-gated scalar differential test for the x86 bits=1 pair kernels.

Code commit: `b4757c3472875fd07e0ecade945c8672dc4be702`

## Scope Notes

This completes the current x86 bits=1 single-candidate and batched weighted byte-LUT coverage. The task text describes bits=1 as sign-popcount, but the production estimator is query-weighted; this slice keeps that semantic contract.

It does not claim bf16, Intel runtime execution, benchmark, recall, or Slice J measurement gates.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits1_pair_matches_scalar_when_available --no-run` completed successfully.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile evidence only.

## Review Focus

- Confirm the pair kernels preserve byte order and bit order relative to scalar bits=1 scoring.
- Confirm batch dispatch now covers both x86 bits=1 kernel variants.
- Confirm odd-cardinality batches fall back through the matching single-candidate x86 kernel.
