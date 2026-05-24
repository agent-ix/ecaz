# Review Request: SPIRE Custom Scan DML Payload Assembly

## Summary

This slice removes the single-use DML UPDATE row-payload helper from SPIRE custom scan execution.

The change:

- inlines JSON payload assembly into the existing `custom_scan_execute_dml_update()` boundary,
- keeps the existing column/value width validation before payload construction,
- preserves per-expression Const/Param evaluation through `custom_scan_dml_update_expr_json_value()`, and
- removes the `custom_scan_dml_update_row_payload_json()` unsafe helper.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2540 -> 2538`
- `rg -n "unsafe" src/am/ec_spire/custom_scan/dml.rs | wc -l`: `28 -> 26`
- `rg -n "unsafe fn" src/am/ec_spire/custom_scan/dml.rs | wc -l`: `11 -> 10`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/dml.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-custom-scan-pg18-pgtest-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

