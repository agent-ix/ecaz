# Review Request: SPIRE CustomScan DML Expression Boundaries

## Summary

Commit `2556b8f4e84d0c38d541fe27d2e6ea8474136d5c` removes two single-use SPIRE CustomScan DML unsafe plan helpers by folding their raw-plan work into the existing `BeginCustomScan` unsafe boundary.

The deleted helpers were only called from `custom_scan_init_dml_exec_state`, which already owns the executor callback contract for a live `CustomScanState` and provider-owned `CustomScan` plan. The remaining UPDATE expression width/offset logic now operates on the typed `CustomScanExprList<'_>` view through `custom_scan_dml_update_value_exprs_from_list`.

This keeps the raw pointer contract at the callback boundary and avoids adding a new safe raw-pointer helper.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2535 -> 2531`
- Deleted:
  - `unsafe fn custom_scan_dml_pk_value_from_plan`
  - `unsafe fn custom_scan_dml_update_value_exprs_from_plan`
  - their internal `unsafe { ... }` blocks
- Added:
  - safe `custom_scan_dml_update_value_exprs_from_list(CustomScanExprList<'_>, expected_count)`

## Validation

See `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/custom_scan/dml.rs src/am/ec_spire/custom_scan/begin_exec.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`

Known warnings only:

- stable-channel rustfmt import grouping warnings
- `src/am/mod.rs` unused SPIRE re-export warning
- Hadamard test-helper dead-code warnings
