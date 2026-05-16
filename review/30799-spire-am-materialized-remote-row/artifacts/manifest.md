# Artifact Manifest: 30799 SPIRE AM Materialized Remote Row

- head SHA: `b8765dd746233f428486b72b4bde8a322d7ba41c`
- code checkpoint SHA: `ec26d05d37794583e59500c46cb0fd8baff48508`
- packet/topic: `30799-spire-am-materialized-remote-row`
- timestamp: `2026-05-10T20:19:07-07:00`
- lane: Task 30 Phase 11.5 Stage D AM SQL delivery proof
- fixture: focused PG18 pgrx loopback plus focused Rust/PG diagnostics tests
- storage format: RaBitQ remote-serving SPIRE indexes with catalog-backed coordinator row materialization
- rerank mode: default SPIRE scan plan with session `ec_spire.max_candidate_rows = 1` in the AM SQL fixture
- isolated/shared surface: isolated coordinator/remote test tables on PG18 loopback; shared extension catalog includes `ec_spire_remote_row_materialization`

## Artifacts

### `pg18-am-materialized-remote-row.log`

- command: `script -q -c "cargo pgrx test pg18 test_ec_spire_prod_scan_am_delivers_materialized_remote_row" review/30799-spire-am-materialized-remote-row/artifacts/pg18-am-materialized-remote-row.log`
- key result lines:
  - `test tests::pg_test_ec_spire_prod_scan_am_delivers_materialized_remote_row ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1600 filtered out`

### `snapshot-diagnostics.log`

- command: `script -q -c "cargo test snapshot_diagnostics --no-default-features --features pg18" review/30799-spire-am-materialized-remote-row/artifacts/snapshot-diagnostics.log`
- key result lines:
  - `test am::ec_spire::diagnostics::tests::snapshot_diagnostics_counts_partition_objects_and_assignments ... ok`
  - `test am::ec_spire::diagnostics::tests::snapshot_diagnostics_counts_degraded_unavailable_without_reading_object ... ok`
  - `test tests::pg_test_ec_spire_active_snapshot_diagnostics_sql ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1598 filtered out`

## Notes

- This packet addresses the first action item in reviewer direction packet `30800`: an end-to-end SQL fixture with an explicit `ec_spire_register_remote_row_materialization(...)` call.
- The fixture also caught and fixed a planner/costing blocker: metadata diagnostics used by PostgreSQL planning must tolerate remote-owned placements by reading the coordinator's published metadata copy, while execution still treats those placements as remote-owned.
