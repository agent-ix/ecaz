# Review Request: SPIRE CustomScan DML Expression Value Evaluators

## Summary

Commit `b45f9f7640bb36a5368ad910320c5443740ee85d` removes two single-use SPIRE CustomScan DML unsafe expression evaluator helpers.

The PK expression evaluation now lives directly inside `custom_scan_init_dml_exec_state`'s existing `BeginCustomScan` unsafe boundary. UPDATE expression JSON conversion now lives inside the existing guarded DML UPDATE executor block that already holds `CustomScanAccessState<'_>`.

This keeps the PostgreSQL executor contracts at the two actual callback-owned boundaries instead of preserving extra unsafe helper boundaries:

- `unsafe fn custom_scan_bigint_expr_value` deleted.
- `unsafe fn custom_scan_dml_update_expr_json_value` deleted.
- No new safe raw-pointer helper was introduced.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2529 -> 2525`
- Deleted both single-use unsafe functions and their internal unsafe blocks.

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
