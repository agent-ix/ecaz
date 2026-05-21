# Review Request: SPIRE CustomScan DML Begin Init

## Summary

This checkpoint reduces duplicated unsafe initialization in SPIRE CustomScan `BeginCustomScan` by centralizing DML mode setup in one helper. The callback now acknowledges the live `CustomScanState` / provider-owned `CustomScan` plan boundary once per DML branch and delegates the repeated plan metadata extraction to `custom_scan_init_dml_exec_state`.

Code commit: `a8da2e9c7d131c7aaa035127f4cc20fde98347fb`

## Scope

- Added `custom_scan_init_dml_exec_state`.
- Deduplicated DML PK-select/update/delete state initialization.
- Kept tuple-payload initialization for PK-select mode while sharing PK value, column-list, update expression, and metadata validation logic.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/custom_scan/begin_exec.rs` | 45 | 38 |

Current packet-local `src/` unsafe ledger: `1917` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1917 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
