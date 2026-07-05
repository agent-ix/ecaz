# Review Request: SPIRE Diagnostics Active Fixture Split

## Summary

Code commit: `a828f2b210e8d43f6cccfbf77b7990cbf74e3751`

This checkpoint extends the Phase 12b.2 `tests/diagnostics.rs` concern file by moving the active snapshot, large-routing diagnostics, and allocator snapshot fixtures from `src/tests/mod.rs` into `src/tests/diagnostics.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved:
  - `test_ec_spire_active_snapshot_diagnostics_sql`
  - `test_ec_spire_large_routing_object_builds_and_scans`
  - `test_ec_spire_allocator_snapshot_sql`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31011`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_active_snapshot_diagnostics_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_allocator_snapshot_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

Both focused PG18 tests passed. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/diagnostics.rs` remains open because top-graph and placement diagnostic fixtures still live outside the concern file.
