# SPIRE Libpq Executor Work Plans

## Summary

This checkpoint adds per-node pre-I/O executor work-plan rows for remote search
and manifest publication.

Changes:

- Adds `ec_spire_remote_search_libpq_executor_work_plan(...)`.
- Composes each remote search dispatch row with bind readiness and executor
  readiness into one work row.
- Reports selected PIDs, bind count/status, dispatch action, next executor
  step, executor status, work action, and effective status.
- Adds `ec_spire_remote_epoch_manifest_libpq_executor_work_plan(...)`.
- Composes manifest publication dispatch rows with bind readiness and executor
  readiness into one work row per remote node.
- Updates active, blocked, and manifest publication PG18 coverage.
- Updates the Phase 7 task note with both executor work-plan surfaces.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `d3fc34cd`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_req_blocked`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `git diff --check`

Result:

- PG18 blocked libpq request filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_blocked`
- PG18 active descriptor catalog filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
- PG18 remote epoch manifest persistence filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`

## Notes

This remains pre-I/O. The work plans identify immediate executor work and
blockers, but they do not resolve secrets, open libpq connections, enter
pipeline mode, send requests, or receive result rows.
