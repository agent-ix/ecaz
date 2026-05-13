# Review Request: SPIRE Diagnostics Fixture Split

## Summary

Code commit: `64756fa05a1983955229f35106c83bc0ea020eea`

This checkpoint starts the Phase 12b.2 `tests/diagnostics.rs` concern file by moving the hierarchy, object, delta, and options snapshot fixtures from `src/tests/mod.rs` into `src/tests/diagnostics.rs`.

The move preserves the textual include style used by the other Phase 12b test splits, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Added `src/tests/diagnostics.rs`.
- Moved:
  - `test_ec_spire_hierarchy_snapshot_sql`
  - `test_ec_spire_object_snapshot_sql`
  - `test_ec_spire_delta_snapshot_sql`
  - `test_ec_spire_delta_snapshot_sql_delete_delta`
  - `test_ec_spire_options_snapshot_sql`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31007`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_hierarchy_snapshot_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_options_snapshot_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.

Both focused tests passed. They emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/diagnostics.rs` remains open because later scan-sanity, health, relation-storage, top-graph, and placement diagnostic fixtures still live outside the concern file.
