# SPIRE Remote Manifest Publication Plan

## Summary

This checkpoint adds a SQL-visible, pre-I/O publication plan for persisted
remote epoch manifests.

Changes:

- Adds `ec_spire_remote_epoch_manifest_publication_plan(...)`.
- Adds `ec_spire_remote_epoch_manifest_publication_summary(...)`.
- Adds `ec_spire_remote_epoch_manifest_publication_contract()`.
- Projects the current manifest plan and persisted manifest catalog into
  per-node publication rows.
- Reports whether the persisted manifest entry exists and still matches the
  current manifest plan.
- Reports `publish_remote_epoch_manifest` with `libpq_pipeline` only when the
  persisted catalog is fresh.
- Reports `persist_remote_epoch_manifest` or `refresh_remote_epoch_manifest`
  when publication is blocked on missing or stale persisted manifest state.
- Aggregates per-node publication rows into one publication decision with
  ready, persistence-required, refresh-required, and blocked counts.
- Reports local-only manifest publication as `not_required` in both catalog and
  publication summaries.
- Publishes the ordered prerequisite/action contract for future manifest
  publication: publish gate, persisted catalog, entry freshness, per-node plan
  readiness, and libpq-pipeline transport.
- Updates the Phase 7 task note with the publication-plan surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `9a801c87`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_epoch_manifest_persist_ready`
- `cargo pgrx test pg18 remote_node_cap_summary_local`
- `cargo pgrx test pg18 remote_epoch_manifest_catalog_summary_missing`
- `cargo pgrx test pg18 remote_phase7_policy_contracts`
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
- PG18 `remote_node_cap_summary_local` filter passed:
  - `pg_test_ec_spire_remote_node_cap_summary_local`
- PG18 `remote_epoch_manifest_catalog_summary_missing` filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_catalog_summary_missing`
- PG18 `remote_phase7_policy_contracts` filter passed:
  - `pg_test_ec_spire_remote_phase7_policy_contracts`
- The test covers ready persisted-manifest publication and stale persisted-entry
  refresh blocking, including the publication summary.
- The local summary test covers local-only `not_required` catalog and
  publication summaries.
- The missing catalog summary test covers the publication summary's
  `persist_remote_epoch_manifest` blocker.
- The Phase 7 policy-contract test covers the manifest publication contract.

## Notes

This remains pre-I/O. The new surface identifies which remote manifest entries
are eligible for future libpq publication, but it does not send manifests to
remote nodes.
