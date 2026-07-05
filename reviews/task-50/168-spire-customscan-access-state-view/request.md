# Review Request: SPIRE CustomScan Access State View

## Summary

This checkpoint adds `CustomScanAccessState<'a>`, a borrowed executor access view for SPIRE CustomScan callbacks. The unsafe constructor validates the live PostgreSQL `ScanState` callback pointer once, and slot access, slot clearing, processed-count accounting, and row-version fetches now go through methods on that view.

Code commit: `3e0d329ce173d69b099ec860cffebef1f18cb86d`

## Scope

- Added a typed SPIRE CustomScan access-state view in `begin_exec.rs`.
- Removed the old raw `ScanState` helper functions for tuple slot, slot clear, processed count, and row-version fetch.
- Updated vector and DML access paths to use the view.
- Updated remote tuple-payload storage to receive the access view instead of a raw `ScanState` pointer.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/custom_scan/begin_exec.rs` | 38 | 25 |
| `src/am/ec_spire/custom_scan/tuple_payload.rs` | 6 | 6 |

Current packet-local `src/` unsafe ledger: `1885` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1885 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
