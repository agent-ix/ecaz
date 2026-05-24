# Review Request: SPIRE CustomScan DML PK Column Boundary

## Summary

Commit `1c35dd3cdd171f460f697a3a60bdf45ea891309d` removes the single-use `unsafe fn custom_scan_dml_pk_column`.

The DML PK SELECT relation/catalog read now lives directly inside `custom_scan_init_dml_exec_state`'s existing `BeginCustomScan` unsafe boundary. That boundary already owns the live `CustomScanState`/`CustomScan` contract and copies all plan-derived metadata into Rust-owned executor state before returning.

No safe raw-pointer helper was introduced.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2525 -> 2523`
- Deleted:
  - `unsafe fn custom_scan_dml_pk_column`
  - its internal caller-side unsafe relation lookup block

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
