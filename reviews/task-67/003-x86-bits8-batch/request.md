# Task 67: x86 bits=8 Batched Scoring

## Summary

This packet extends the x86 bits=8 work from packet 002 into the batched `estimate_ip_batch` path:

- `estimate_ip_batch_impl` now dispatches `Avx512Bits8` to `estimate_ip_batch_avx512_bits8`.
- `estimate_ip_batch_impl` now dispatches `Avx2Bits8` to `estimate_ip_batch_avx2_bits8`.
- Added paired AVX-512F and AVX2+FMA bits=8 kernels that reuse each query scale/offset vector load for two candidate codes.
- Odd-sized batches fall through to the single-candidate x86 bits=8 kernel for the final tail candidate.

Code commit: `e6439ec81ccc84c6907b8268f0510ac402acc684`

## Scope Notes

This completes the Task 67 bits=8 single-candidate and batched x86 implementation path. It does not complete bits=1, bits=4, bf16 evaluation, or Intel benchmark/recall gates.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::bits8_batch_estimator_matches_scalar_order --no-run` completed successfully.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile evidence only.

## Review Focus

- Confirm the batch pair kernels preserve the same `code_byte * query_scale + query_offset` formula as packet 002.
- Confirm the pair loop handles odd batch tails correctly.
- Confirm the target-feature functions remain behind runtime-selected dispatch arms.
