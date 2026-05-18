# Review Request: SPIRE Boundary Replica Remote Identity

## Summary

Closes the Phase 12.7 row:

> Add remote-node multi-instance proof that boundary replicas carry the same
> global original-vector identity across leaves, stores, and remotes.

This extends `ec_spire_index_boundary_replica_identity_snapshot(index_oid)` so
the diagnostic can read coordinator metadata copies for synthetic remote
placements while still reporting the original placement node/local-store span.
That lets the fixture prove identity propagation across remote placement
metadata before live remote object reads exist.

The PG18 fixture now:

- builds an index with `source_identity = 'include'`,
  `boundary_replica_count = 1`, and two local stores;
- rewrites one leaf placement to remote node/local store `2`;
- asserts four ready global identity rows remain present;
- asserts primary and boundary-replica assignment counts remain one each per
  source identity; and
- asserts at least one ready global identity spans node IDs `0..2`.

## Files

- `src/am/ec_spire/root/diagnostics.rs`
- `src/lib.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `git diff --check e1762b5f^ e1762b5f`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_identity_snapshot_global_ids`

## Reviewer Focus

- Confirm the metadata-read treatment of synthetic remote placements is the
  right boundary for this readiness proof.
- Confirm the diagnostic still reports the original remote `node_id` and
  `local_store_id` rather than the local metadata-read override.
- Confirm the tracker row closure is scoped to identity/metadata proof only,
  not live remote object reads or degraded replica freshness.
