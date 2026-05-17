# SPIRE Manifest Publication Gate Summary

## Summary

This checkpoint adds a final pre-I/O gate summary for remote manifest
publication.

Changes:

- Adds `ec_spire_remote_epoch_manifest_publication_gate_summary(...)`.
- Composes publication readiness, libpq request count, libpq dispatch count,
  libpq receive count, receive readiness, executor status, next blocker, and
  effective status into one row.
- Extends local-only and distributed manifest publication PG18 coverage for the
  new composed gate.
- Updates the Phase 7 task note with the publication gate surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `e182f943`

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

This remains pre-I/O. The gate identifies the next unresolved publication
blocker and counts the planned libpq request/dispatch/receive stages, but it
does not execute libpq transport or apply remote manifest state.
