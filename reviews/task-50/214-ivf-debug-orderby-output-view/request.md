# Review Request: IVF Debug ORDER BY Output View

## Summary

This slice removes the inline unsafe debug reads of `IndexScanDescData::xs_orderbynulls` and `xs_orderbyvals` from the IVF scan debug helper call sites.

The change adds a `pg_test`/test-only `IvfScanDescView::first_orderby_output()` method that:

- checks the PostgreSQL-owned ORDER BY null/value pointers in one boundary,
- copies the first null flag and score value immediately,
- keeps the `f32::from_datum` conversion inside that same boundary, and
- exposes safe copied Rust values to `debug_scan_first_orderby_is_null()` and `debug_scan_first_orderby_score()`.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2555 -> 2554`
- `rg -n "unsafe" src/am/ec_ivf/scan.rs | wc -l`: `46 -> 45`
- `rg -n "unsafe fn" src/am/ec_ivf/scan.rs | wc -l`: `12 -> 12`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_ivf/scan.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-ec-ivf-pg18-pgtest-no-run.log`: `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

