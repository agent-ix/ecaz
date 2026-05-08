# SPIRE Remote Manifest Publication Plan

## Summary

This checkpoint adds a SQL-visible, pre-I/O publication plan for persisted
remote epoch manifests.

Changes:

- Adds `ec_spire_remote_epoch_manifest_publication_plan(...)`.
- Projects the current manifest plan and persisted manifest catalog into
  per-node publication rows.
- Reports whether the persisted manifest entry exists and still matches the
  current manifest plan.
- Reports `publish_remote_epoch_manifest` with `libpq_pipeline` only when the
  persisted catalog is fresh.
- Reports `persist_remote_epoch_manifest` or `refresh_remote_epoch_manifest`
  when publication is blocked on missing or stale persisted manifest state.
- Updates the Phase 7 task note with the publication-plan surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `8e268f56`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_epoch_manifest_persist_ready`
- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `git diff --check`

Result:

- PG18 `remote_epoch_manifest_persist_ready` filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`
- The test covers ready persisted-manifest publication and stale persisted-entry
  refresh blocking.

## Notes

This remains pre-I/O. The new surface identifies which remote manifest entries
are eligible for future libpq publication, but it does not send manifests to
remote nodes.
