# Task 67 Review Request: X86 Differential Scaffold

## Summary

This packet adds a test-facing RaBitQ sum-query-dequant kernel registry in
`src/quant/rabitq.rs` for Task 67 differential coverage. The registry exposes
private scalar, AVX2, AVX-512, and gated AVX-512 BF16 kernel entry points
through fixture-driven `*_for_test` wrappers so future tests and benchmark
audits can enumerate the optimized kernels without stringly-typed private
function knowledge.

## Code Under Review

- Commit: `a22677eb8a0822196e1af921e819ec0c289560de`
- File: `src/quant/rabitq.rs`

## Behavior Added

- Adds `bench_api::sum_query_dequant_kernels_for_test()`.
- Registers scalar bits1/bits4/bits8 entries.
- Registers x86_64 AVX2 and AVX-512 bits1/bits4/bits8 entries with explicit
  declared feature requirements.
- Registers the AVX-512 BF16 bits4 entry when `rabitq-bf16` is enabled.
- Adds compile-covered unit tests that assert the expected registry entries and
  compare feature-available kernels against scalar fixtures for bits 1, 4, and
  8 at odd and aligned dimensions.

## Validation

See `artifacts/validation.log` and `artifacts/manifest.md`.

- `cargo fmt` passed.
- `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_registers_expected_kernels --no-run` passed.
- `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available --no-run` passed.
- `git diff --check` passed.

Runtime execution was not claimed for this packet because the local runtime test
path remains blocked by the existing PostgreSQL `LockBuffer` symbol issue. This
host also lacks AVX-512, so AVX-512 runtime acceptance remains pending on
suitable Intel hardware.
