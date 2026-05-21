# Review Request: SPIRE CustomScan Plan-Private View

## Summary

This checkpoint reduces the SPIRE CustomScan plan-private unsafe surface by adding a typed `CustomScanPlanPrivate` view for provider-owned PostgreSQL `List` metadata. The view validates the live list once, captures length/tag metadata, and exposes safe bounded accessors for OID, node, string, and u32 reads.

Code commit: `e6fd859ef0ac90a238039b123f2db34055b3704f`

## Scope

- Added `CustomScanPlanPrivate<'a>` in `src/am/ec_spire/custom_scan/plan_private.rs`.
- Moved repeated `List` length/tag/nth/string/u32 decoding into safe methods on the typed view.
- Converted plan-private DML column-list and PK-column readers from raw-list `unsafe fn` helpers to safe functions over the view.
- Updated DML `top_k` extraction to consume the same plan-private view.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 49 | 32 |
| `src/am/ec_spire/custom_scan/dml.rs` | 25 | 24 |

Current packet-local `src/` unsafe ledger: `1924` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1924 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
