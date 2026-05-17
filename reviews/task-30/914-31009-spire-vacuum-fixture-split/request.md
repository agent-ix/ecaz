# Review Request: SPIRE Vacuum Fixture Split

## Summary

Code commit: `2674571c2c5fb3e9b7300944a6ed1ea5c70ab1b2`

This checkpoint starts the Phase 12b.2 `tests/vacuum.rs` concern file by moving the epoch cleanup, epoch snapshot, and maintenance-run fixture block from `src/tests/mod.rs` into `src/tests/vacuum.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Added `src/tests/vacuum.rs`.
- Moved:
  - `test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql`
  - `test_ec_spire_epoch_snapshot_sql`
  - `test_ec_spire_maintenance_run_empty_sql`
  - `test_ec_spire_locked_maintenance_run_plan_no_write_sql`
  - `test_ec_spire_maintenance_run_no_candidate_sql`
  - `test_ec_spire_recursive_maintenance_run_rejected`
  - `test_ec_spire_maintenance_run_merge_publish_sql`
  - `test_ec_spire_maintenance_run_split_publish_sql`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31009`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_maintenance_run_split_publish_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

Both focused PG18 tests passed. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/vacuum.rs` remains open because later SQL VACUUM and concurrent insert/vacuum fixtures still live outside the concern file.
