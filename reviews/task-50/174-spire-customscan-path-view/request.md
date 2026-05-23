# Review Request: SPIRE CustomScan Path View

## Scope

This packet reviews commit `07c1dead3ecb5c485deb233ba5d9d5959aa4724e` (`Add SPIRE CustomScan path view`).

The slice adds `CustomScanPath<'a>` and `CustomScanPathPlanFields` in `src/am/ec_spire/custom_scan/plan_private.rs`, then uses the view in `PlanCustomPath` construction.

## Unsafe Burndown

Removed the old raw CustomPath helpers:

- `custom_scan_mode_from_path`
- `custom_scan_index_oid_from_path`

The replacement view:

- validates the provider-owned `CustomPath` once at the PlanCustomPath callback boundary;
- exposes safe `mode()` and `index_oid()` metadata readers;
- exposes one checked `plan_fields()` accessor for copied cost/width/scanrelid/flags metadata;
- removes direct `(*best_path)` dereferences from vector and DML CustomScan plan construction.

Unsafe ledger movement:

- previous packet 173 ledger: `1853`
- packet 174 ledger: `1852`
- net reduction: `1`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_spire/custom_scan/plan_private.rs`: `17 -> 16`
- `src/am/ec_spire/custom_scan/planner.rs`: remains `22`, with raw `best_path` reads now behind `CustomScanPath`.

## Validation

Packet-local artifacts are under `reviews/task-50/174-spire-customscan-path-view/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that `CustomScanPath<'a>` does not overstate the lifetime guarantee from the PlanCustomPath callback, and that `plan_fields()` correctly null-checks `parent` before reading `scanrelid` while preserving the previous `pathtarget` null behavior for plan width.
