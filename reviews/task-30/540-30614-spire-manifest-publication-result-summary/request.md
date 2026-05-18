# SPIRE Manifest Publication Result Summary

## Scope

This packet adds the final operator-facing summary row for remote epoch
manifest publication before real libpq socket I/O exists.

Code checkpoint: `6250bfdd` (`Add SPIRE manifest publication result summary`)

## Changes

- Adds `ec_spire_remote_epoch_manifest_publication_result_summary(...)`.
- Composes the publication gate into a result source:
  - `not_required` for local-only indexes.
  - `pending_libpq_executor` for distributed manifests waiting on the future
    libpq executor.
  - `remote_manifest_validation_result` for future ready validation results.
  - `blocked` for pre-publication blockers.
- Carries publication entry count, libpq receive counts,
  payload-validation result status, next blocker, and effective status into the
  final summary.
- Extends local-only and distributed manifest publication PG18 coverage.
- Updates the Phase 7 task note with the new result-summary surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_node_cap_summary_local`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `git diff --check`

## Notes

This remains pre-I/O. The result summary reports the exact point where manifest
publication stops today, but it does not execute libpq transport or apply
remote manifest state durably.
