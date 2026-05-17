# Review Request: SPIRE Build Fixture Split

## Summary

Code commit: `82bb7fffab004af04ab1ce55eb232d949d36e9a5`

This checkpoint starts the Phase 12b.2 `tests/build.rs` concern file by moving the initial boundary-replica, recursive boundary-replica, and PQ-FastScan populated build-deferral fixtures from `src/tests/mod.rs` into `src/tests/build.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Added `src/tests/build.rs`.
- Moved:
  - `test_ec_spire_boundary_replica_build_writes_and_dedupes_scan`
  - `test_ec_spire_recursive_boundary_replica_build_dedupes`
  - `test_ec_spire_pq_fastscan_populated_build_reports_deferral`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31008`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_boundary_replica_build_writes_and_dedupes_scan -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_pq_fastscan_populated_build_reports_deferral -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.

Both focused tests passed. The PQ-FastScan fixture passed via its expected `should_panic` path. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/build.rs` remains open because later populated-build, multistore, recursive-fanout, and top-graph build fixtures still live outside the concern file.
