# Task 67: x86 AVX-512 BF16 RaBitQ Kernel

## Summary

This packet lands the feature-gated x86 AVX-512 BF16 bits=4 scoring path:

- `Avx512Bf16Bits4` now dispatches to `sum_query_dequant_avx512_bf16_bits4` behind AVX-512F+BF16 runtime selection and the existing `rabitq-bf16` Cargo feature.
- The kernel uses `_mm512_dpbf16_ps` over prepared bf16 query/dequant mirrors and preserves scalar bf16 truncation semantics in the tail.
- Added a feature-gated differential test against bf16-rounded scalar math.
- Tightened x86 bf16 slot selection to require the prepared bf16 query mirror to match `dimensions`.

Code commit: `f909b16ded2b276a10c952aab1516e00c1759b56`

## Scope Notes

This is implementation/readiness for Task 67 Slice I. It does not claim Intel bf16 performance is a win, does not enable `rabitq-bf16` by default, and does not satisfy the required Intel runtime benchmark/recall gates.

## Validation

See `artifacts/validation.log`.

- `cargo fmt` completed successfully, with the repo's usual stable-rustfmt warnings.
- `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits4_matches_scalar_when_available --no-run` completed successfully.
- A standalone `rustc` probe for the AVX-512 BF16 intrinsic shape compiled successfully.
- Full feature-enabled cargo validation with `--features rabitq-bf16` could not be completed in this environment: cargo repeatedly spun without spawning `rustc` or producing diagnostics and was stopped after multiple attempts.
- Runtime unit-test execution remains blocked in this local environment by unresolved PostgreSQL symbol `LockBuffer`, so this packet contains compile/readiness evidence only.

## Review Focus

- Confirm the `_mm512_dpbf16_ps` lane grouping matches the bf16 scalar reference: 32 bf16 inputs reduce to 16 f32 accumulator lanes.
- Confirm the bf16 dispatch remains opt-in and runtime gated by AVX-512F+BF16.
- Confirm the validation limitation is acceptable for this slice, or request a follow-up on an Intel host that can run the feature-enabled cargo and runtime gates.
