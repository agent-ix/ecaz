# 30774 - SPIRE Remote Manifest Freshness Diagnostics

## Summary

This packet reviews commit `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
(`Expose SPIRE remote manifest freshness diagnostics`).

The slice adds `ec_spire_remote_epoch_manifest_freshness(...)`, a node-level
Stage E assertion surface for remote epoch manifests.

Changes:

- Composes the current manifest plan, persisted manifest entry state, catalog
  summary, and publication plan into one per-remote-node row.
- Reports `freshness_status` and `next_action` so fixtures can distinguish
  ready manifests from missing persistence and stale persisted entries before
  remote fanout.
- Registers the surface in
  `ec_spire_remote_operator_entrypoint_contract()` as
  `stage_e_manifest_freshness_assertion`.
- Updates Phase 11 docs to state that this covers node-level manifest
  freshness while per-boundary-replica fixture evidence remains open.
- Extends the existing PG18 manifest persistence test to verify ready and
  intentionally stale persisted-entry cases.

## Key Files

- `src/lib.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

- `cargo fmt --check`
- `git diff --check -- src/lib.rs src/am/ec_spire/root/remote_candidates.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`

## Review Focus

- Is `ec_spire_remote_epoch_manifest_freshness(...)` a reasonable composition
  over the existing plan/catalog/publication surfaces, or should it live lower
  in the Rust snapshot layer?
- Are the `freshness_status` and `next_action` categories sufficient for Stage
  E fixture assertions?
- Does the task/docs wording avoid overclaiming boundary-replica fixture
  evidence?
