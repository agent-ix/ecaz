# SPIRE Remote Manifest Catalog Summary

## Summary

This checkpoint adds a SQL-visible health summary over the persisted
distributed epoch manifest catalog.

Changes:

- Adds `ec_spire_remote_epoch_manifest_catalog_summary(...)`.
- Compares the current manifest decision against persisted manifest rows for
  the active epoch.
- Reports whether persistence is not required, blocked upstream, missing,
  stale, or ready.
- Exposes current included-node/placement counts alongside persisted
  manifest/entry/placement counts.
- Extends the ready persistence test to prove the summary reports `ready` after
  persistence.
- Adds missing-persistence coverage for a distributed-ready manifest that has
  not yet been written.
- Updates the Phase 7 task note with the catalog summary surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `751158e7`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_epoch_manifest_catalog_summary`
- `cargo pgrx test pg18 remote_epoch_manifest_persist`
- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `git diff --check`

Result:

- PG18 `remote_epoch_manifest_catalog_summary` filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_catalog_summary_missing`
- PG18 `remote_epoch_manifest_persist` filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`
  - `pg_test_ec_spire_remote_epoch_manifest_persist_blocked`
- The ready test confirms the summary reaches `ready` after persistence; the
  missing test confirms distributed-ready manifests report
  `requires_remote_epoch_manifest_persistence` before persistence.

## Notes

This remains coordinator-side catalog health. It does not push manifests to
remote nodes or open libpq transport.
