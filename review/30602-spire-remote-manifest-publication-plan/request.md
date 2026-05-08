# SPIRE Remote Manifest Publication Plan

## Summary

This checkpoint adds a SQL-visible, pre-I/O publication plan for persisted
remote epoch manifests.

Changes:

- Adds `ec_spire_remote_epoch_manifest_publication_plan(...)`.
- Adds `ec_spire_remote_epoch_manifest_publication_summary(...)`.
- Adds `ec_spire_remote_epoch_manifest_publication_contract()`.
- Adds `ec_spire_remote_epoch_manifest_libpq_request_plan(...)`.
- Adds `ec_spire_remote_epoch_manifest_libpq_request_summary(...)`.
- Adds `ec_spire_remote_epoch_manifest_libpq_parameter_contract()`.
- Adds `ec_spire_remote_epoch_manifest_libpq_result_contract()`.
- Adds `ec_spire_remote_epoch_manifest_libpq_executor_step_contract()`.
- Adds `ec_spire_remote_epoch_manifest_payload_plan(...)`.
- Adds `ec_spire_remote_epoch_manifest_payload_summary(...)`.
- Adds `ec_spire_apply_remote_epoch_manifest(...)`.
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
- Exposes `publication_executor_status` and `publication_executor_next_step`
  on the publication summary so ready manifests still show the libpq executor
  handoff.
- Exposes a per-node future libpq request envelope for ready manifest
  publication, including descriptor-backed secret/index metadata, payload
  source/format, SQL template, parameter/result counts, and executor handoff
  status.
- Aggregates request-plan rows into a pre-I/O request summary, and publishes the
  bind-parameter, apply-result, and executor-step contracts for the future
  libpq manifest publication executor.
- Materializes catalog-backed JSONB manifest payloads for ready publication
  requests and validates those payloads on the remote apply surface before any
  durable remote apply/write path exists.
- Reports local-only manifest publication as `not_required` in both catalog and
  publication summaries, with no libpq request rows.
- Publishes the ordered prerequisite/action contract for future manifest
  publication: publish gate, persisted catalog, entry freshness, per-node plan
  readiness, and libpq-pipeline transport.
- Updates the Phase 7 task note with the publication-plan surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `977b8f37`

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
- The ready persisted-manifest test covers publication readiness, stale
  persisted-entry refresh blocking, the publication summary, executor handoff,
  request-plan envelope, ready request-summary counts, JSONB payload
  materialization, successful remote apply validation, and an epoch-mismatch
  apply blocker.
- The local summary test covers local-only `not_required` catalog and
  publication summaries with no executor handoff, no request rows, and a
  `not_required` request/payload summary.
- The missing catalog summary test covers the publication summary's
  `persist_remote_epoch_manifest` blocker.
- The Phase 7 policy-contract test covers the manifest publication contract,
  manifest libpq parameter contract, manifest libpq result contract, and
  manifest libpq executor-step contract.

## Notes

This remains pre-I/O. The new surfaces identify which remote manifest entries
are eligible for future libpq publication, expose the request and payload shape,
and define the executor/apply-result contracts, but they do not send manifests
to remote nodes or persist remote apply state.
