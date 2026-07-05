# Review Request: SPIRE Diagnostics Final Fixture Split

## Summary

Code commit: `bb36316c8ef84cb1c1ab4933433be59a7edb77f9`

This checkpoint closes the Phase 12b.2 `tests/diagnostics.rs` concern file by moving the remaining top-graph snapshot and boundary-replica placement diagnostics fixtures from `src/tests/mod.rs` into `src/tests/diagnostics.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved:
  - `test_ec_spire_top_graph_snapshot_sql`
  - `test_ec_spire_boundary_replica_placement_diagnostics_sql`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31012` and mark `tests/diagnostics.rs` closed.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_top_graph_snapshot_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_boundary_replica_placement_diagnostics_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

Both focused PG18 tests passed. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/diagnostics.rs` is closed. Other concern files remain open under the tracker.
