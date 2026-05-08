# SPIRE Coordinator Gate Receive Status

## Summary

This checkpoint threads remote search receive state into the top-level
coordinator gate summary.

Changes:

- Adds `libpq_receive_count` and `libpq_receive_status` to
  `ec_spire_remote_search_coordinator_gate_summary(...)`.
- Computes receive count/status from the same receive-plan rows used by the
  SQL-visible receive boundary.
- Extends coordinator-gate PG18 coverage for local-only and descriptor-blocked
  remote plans.
- Updates the Phase 7 task note so the coordinator gate explicitly includes
  receive readiness.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `669a6060`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_search_coordinator_gate_summary`
- `git diff --check`

Result:

- PG18 coordinator gate summary filter passed:
  - `pg_test_ec_spire_remote_search_coordinator_gate_summary`

## Notes

This remains pre-I/O. The coordinator gate now carries receive readiness, but
remote candidate batches still require the future libpq executor before merge
can ingest remote result rows.
