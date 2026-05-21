# Review Request: SPIRE Planner Relation View

## Summary

This checkpoint adds `CustomScanPlannerRel<'a>`, a borrowed planner callback view for SPIRE CustomScan planning. The unsafe constructor validates the live PostgreSQL `PlannerInfo` / `RelOptInfo` pointers, and the repeated planner relation, sort target, index-list, target-width, and path cost reads now go through methods on that view.

Code commit: `f84dba492bc37cf310225aabd81d222ea72c4693`

## Scope

- Added a typed SPIRE planner relation view in `cost_helpers.rs`.
- Moved vector ORDER BY query extraction and index-list iteration behind the view.
- Updated CustomScan path construction in `planner.rs` to use the view instead of repeated raw root/rel reads.
- Kept DML handoff and CustomPath allocation behavior unchanged.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/custom_scan/cost_helpers.rs` | 33 | 25 |
| `src/am/ec_spire/custom_scan/planner.rs` | 33 | 22 |

Current packet-local `src/` unsafe ledger: `1898` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1898 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
