# Task 67: x86 bits=4 RaBitQ Batch

## Summary

This packet lands x86 batched bits=4 RaBitQ scoring:

- `estimate_ip_batch_impl` now dispatches `Avx512Bits4` to `estimate_ip_batch_avx512_bits4`.
- `estimate_ip_batch_impl` now dispatches `Avx2Bits4` to `estimate_ip_batch_avx2_bits4`.
- Added AVX-512F+BW and AVX2+FMA pair kernels that reuse query loads while scoring two candidate codes.
- Added a scalar-order batch estimator test for bits=4.

Code commit: `9747b56627dd5564133cf15592433d552427e7a6`

## Scope Notes

This completes the current x86 bits=4 single-candidate and batched scoring coverage. It does not claim bits=1, bf16, Intel runtime execution, benchmark, recall, or Slice J measurement gates.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::bits4_batch_estimator_matches_scalar_order --no-run` completed successfully.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile evidence only.

## Review Focus

- Confirm the bits=4 batch dispatch selects the x86 pair kernels only for the matching kernel variants.
- Confirm the pair kernels preserve scalar nibble decode order for both candidates.
- Confirm the scalar tail and odd-cardinality fallback keep batch results identical to scalar order.
