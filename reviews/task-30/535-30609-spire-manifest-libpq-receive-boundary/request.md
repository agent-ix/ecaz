# SPIRE Manifest Libpq Receive Boundary

## Summary

This checkpoint adds the manifest publication receive/result boundary before
real libpq socket I/O exists.

Changes:

- Adds `ec_spire_remote_epoch_manifest_libpq_receive_plan(...)`.
- Exposes per-node expected result-column count, validator function, result
  action, result contract, and effective status for manifest payload-validation
  results.
- Adds `ec_spire_remote_epoch_manifest_libpq_receive_summary(...)`.
- Aggregates receive rows into ready/blocked counts, expected result contract
  metadata, executor status, next executor step, and effective status.
- Extends local-only and distributed manifest publication PG18 coverage for the
  new receive boundary.
- Updates the Phase 7 task note with the manifest receive-plan and summary
  surfaces.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `38c18677`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_node_cap_summary_local`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `git diff --check`

Result:

- PG18 local-only remote capability summary filter passed:
  - `pg_test_ec_spire_remote_node_cap_summary_local`
- PG18 remote epoch manifest persistence filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`

## Notes

This remains pre-I/O. The receive boundary names the expected result contract
and validation action, but it does not read libpq result rows or apply remote
manifest state durably.
