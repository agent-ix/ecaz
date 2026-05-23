# Review Request: SPIRE Custom Scan UPDATE Datum JSON

## Summary

This slice removes the SPIRE custom scan DML UPDATE Datum-to-JSON helper boundary.

The change:

- folds non-null UPDATE Datum JSON conversion into the existing `custom_scan_dml_update_expr_json_value()` expression-evaluation boundary,
- keeps Const/Param node dispatch, `exprType`, and PostgreSQL type output calls in one callback-local unsafe block, and
- removes the `custom_scan_dml_update_datum_json_value()` unsafe helper.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2538 -> 2536`
- `rg -n "unsafe" src/am/ec_spire/custom_scan/dml.rs | wc -l`: `26 -> 24`
- `rg -n "unsafe fn" src/am/ec_spire/custom_scan/dml.rs | wc -l`: `10 -> 9`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/dml.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-custom-scan-pg18-pgtest-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

