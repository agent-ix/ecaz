# SPIRE Remote Search Receive Summary

## Summary

This checkpoint adds a one-row receive gate for remote search candidate batches.

Changes:

- Adds `ec_spire_remote_search_receive_summary(...)`.
- Aggregates receive-plan rows into ready/blocked receive counts, remote PID
  counts, blocked PID counts, expected result-column count, receive validator,
  row-locator policy, and effective status.
- Extends local-only and blocked remote receive PG18 coverage for the new
  summary.
- Updates the Phase 7 task note with the receive-summary surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `f431519b`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_req_local`
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_plan_blocked`
- `git diff --check`

Result:

- PG18 local-only libpq request filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_local`
- PG18 blocked remote search receive filter passed:
  - `pg_test_ec_spire_remote_search_receive_plan_blocked`

## Notes

This remains pre-I/O. The summary names the expected receive contract and
blocks remote result ingestion until the libpq executor returns real candidate
batches.
