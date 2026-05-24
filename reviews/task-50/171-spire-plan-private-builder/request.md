# Review Request: SPIRE Plan Private Builder

## Summary

This checkpoint replaces repeated unsafe PostgreSQL list-building calls in SPIRE CustomScan DML plan-private construction with `CustomScanPlanPrivateBuilder`. The unsafe constructor captures the active planner memory-context invariant once; appending strings and counted column lists is then safe on the builder.

Code commit: `01ff7a455bf32404faa5e4508c6a93335fcc1a6a`

## Scope

- Added `CustomScanPlanPrivateBuilder`.
- Replaced repeated `custom_scan_lappend_string` / `custom_scan_lappend_counted_column_list` unsafe calls.
- Removed the old lappend helper functions.
- Kept plan-private list layout unchanged.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 31 | 25 |

Current packet-local `src/` unsafe ledger: `1866` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1866 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
