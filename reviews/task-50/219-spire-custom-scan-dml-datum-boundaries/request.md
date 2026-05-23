# Review Request: SPIRE Custom Scan DML Datum Boundaries

## Summary

This slice removes two single-purpose unsafe Datum decode helpers from SPIRE custom scan DML handling.

The change:

- folds ORDER BY real-array Datum decoding into the existing `custom_scan_query_from_plan()` boundary,
- removes the single-use `custom_scan_query_values_from_datum()` helper,
- inlines bigint PK Datum decoding inside the existing expression-node dispatch boundary, and
- removes the `custom_scan_bigint_datum_value()` helper.

This keeps Const/Param NodeTag checks and Datum decoding in the same callback-local boundary.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2544 -> 2540`
- `rg -n "unsafe" src/am/ec_spire/custom_scan/dml.rs | wc -l`: `32 -> 28`
- `rg -n "unsafe fn" src/am/ec_spire/custom_scan/dml.rs | wc -l`: `13 -> 11`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/dml.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-custom-scan-pg18-pgtest-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

