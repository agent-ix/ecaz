# Review Request: SPIRE CustomScan Dead Helper Trim

## Summary

This checkpoint removes dead SPIRE CustomScan unsafe helpers left behind by earlier typed plan-private/list view work. The removed helpers had no remaining callers, so keeping them would have left unnecessary residual unsafe surface.

Code commit: `4516b714814374e6d20f06a94ae34eae08214681`

## Scope

- Removed unused `custom_scan_list_tag`.
- Removed unused `custom_scan_list_nth_oid`.
- Removed unused `custom_scan_string_node_value`.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/custom_scan/cost_helpers.rs` | 25 | 22 |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 32 | 31 |

Current packet-local `src/` unsafe ledger: `1872` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1872 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
