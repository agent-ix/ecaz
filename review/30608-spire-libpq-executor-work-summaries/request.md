# SPIRE Libpq Executor Work Summaries

## Summary

This checkpoint adds one-row executor work summaries for remote search and
manifest publication.

Changes:

- Adds `ec_spire_remote_search_libpq_executor_work_summary(...)`.
- Aggregates remote search executor work rows into ready/blocked work counts,
  remote PID counts, blocked PID counts, next executor step, executor status,
  and effective status.
- Adds `ec_spire_remote_epoch_manifest_libpq_executor_work_summary(...)`.
- Aggregates manifest publication executor work rows into ready/blocked work
  counts, bind-ready counts, next executor step, executor status, and effective
  status.
- Extends blocked, active descriptor, and manifest publication PG18 coverage to
  assert the new summary gates.
- Updates the Phase 7 task note with both executor work-summary surfaces.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `45353c0ecb1df86b5d56474091045878dd64c768`

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

This remains pre-I/O. The summary functions collapse existing plan/readiness
surfaces into coordinator gates, but they do not resolve secrets, open libpq
connections, enter pipeline mode, send requests, or receive result rows.
