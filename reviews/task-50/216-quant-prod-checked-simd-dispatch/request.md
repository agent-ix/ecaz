# Review Request: Quantized Product Checked SIMD Dispatch

## Summary

This slice tightens the RaBitQ-adjacent quantized-product scoring dispatch in `src/quant/prod.rs`.

The change:

- adds checked AVX2/FMA and NEON helper methods for split-part QJL scoring,
- adds checked AVX2/FMA and NEON helper methods for 3-bit MSE-code scoring,
- routes production and test SIMD call sites through those checked helpers, and
- falls back to scalar scoring if a forced SIMD backend is selected but the CPU feature is not actually available.

That keeps the target-feature `unsafe` calls in four local dispatch boundaries instead of repeating them at every caller.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2552 -> 2548`
- `rg -n "unsafe" src/quant/prod.rs | wc -l`: `17 -> 13`
- `rg -n "unsafe fn" src/quant/prod.rs | wc -l`: `5 -> 5`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/quant/prod.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-quant-prod-pg18-bench-no-run.log`: `cargo test --lib quant::prod --no-default-features --features pg18,bench --no-run` passed.

