# Review Request: SPIRE Diagnostics Sanity Fixture Split

## Summary

Code commit: `2325892db352ccc91d6f90e787f551b4a0f47779`

This checkpoint extends the Phase 12b.2 `tests/diagnostics.rs` concern file by moving the scan-sanity, health, and relation-storage snapshot fixtures from `src/tests/mod.rs` into `src/tests/diagnostics.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved:
  - `test_ec_spire_scan_sanity_snapshot_sql`
  - `test_ec_spire_health_snapshot_sql`
  - `test_ec_spire_relation_storage_snapshot_sql`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31010`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_scan_sanity_snapshot_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_relation_storage_snapshot_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

Both focused PG18 tests passed. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/diagnostics.rs` remains open because top-graph, active/allocator, and placement diagnostic fixtures still live outside the concern file.
