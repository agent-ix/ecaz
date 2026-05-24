# Task 50 Packet 209: SPIRE Custom Scan Expr List View

## Summary

This slice consolidates SPIRE custom-scan DML expression-list access behind a typed local view.

Updated surface:

- `src/am/ec_spire/custom_scan/dml.rs` now uses `CustomScanExprList<'_>` for provider-owned `CustomScan.custom_exprs` access.
- The previous raw helper chain `custom_scan_custom_exprs`, `custom_scan_expr_from_plan`, and `custom_scan_expr_from_exprs` was removed.
- ORDER BY query expression, DML PK expression, and UPDATE value expression extraction now use the same bounds-checked expression-list view.

The remaining unsafe in this file is still around live PostgreSQL callback state, executor expression evaluation, datum decoding, tuple payload reads, and type I/O lookup.

## Counts

- `src`: `2566 -> 2564` unsafe references.
- `src/am/ec_spire/custom_scan/dml.rs`: `34 -> 32` unsafe references.
- `src/am/ec_spire/custom_scan/dml.rs`: `15 -> 13` unsafe function contracts.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/custom_scan/dml.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
