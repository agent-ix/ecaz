# Review Request: SPIRE Boundary Replica Manifest Freshness

## Summary

Closes the Phase 12.7 row:

> Add boundary-replica manifest freshness fixtures using
> `ec_spire_remote_epoch_manifest_freshness()`.

This adds `test_ec_spire_boundary_replica_manifest_freshness_sql`, a focused
PG18 fixture that runs the freshness surface against a boundary-replica index
with global source identity.

The fixture:

- builds an index with `source_identity = 'include'` and
  `boundary_replica_count = 1`;
- rewrites one leaf placement to remote node/local store `2`;
- confirms `ec_spire_index_boundary_replica_identity_snapshot(...)` still
  reports at least one ready global identity spanning local and remote nodes;
- confirms freshness reports
  `requires_remote_epoch_manifest_persistence` before persistence;
- persists the manifest and confirms freshness reports `ready`; and
- drifts the persisted catalog entry and confirms freshness reports
  `stale_remote_epoch_manifest` with next action
  `refresh_remote_epoch_manifest`.

## Files

- `src/lib.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `git diff --check e7dc3956^ e7dc3956`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_manifest_freshness_sql`

## Reviewer Focus

- Confirm this fixture covers the intended boundary-replica freshness states:
  missing persisted manifest, ready persisted manifest, and stale persisted
  manifest.
- Confirm pairing the freshness surface with the boundary identity snapshot is
  enough to prove the fixture is actually exercising boundary-replica metadata.
- Confirm the tracker row closure does not imply live remote object reads or
  degraded replica-placement diagnostics are complete.
