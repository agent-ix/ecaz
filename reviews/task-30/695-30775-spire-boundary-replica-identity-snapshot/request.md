# 30775 - SPIRE boundary replica identity snapshot

Review commit: `f607c37fec14ec2c92d25006e884adf2805f5268`

## Summary

This slice adds `ec_spire_index_boundary_replica_identity_snapshot(index_oid)`,
an index-level diagnostic for the Phase 11 identity requirement that boundary
replicas carry the same original-vector identity as their primary assignment.

The diagnostic reads the active epoch placement directory, scans available leaf
and delta assignment rows, groups by `vec_id`, and returns only groups with at
least one boundary replica. Each row reports:

- whether the ID is `global`, `node_local`, or `invalid`;
- assignment, primary, boundary-replica, and delta-insert counts;
- leaf PID span, remote/local node span, and local-store span;
- status and recommendation.

`ready` is reserved for global IDs with one primary and at least one boundary
replica. Node-local IDs remain visible as `local_scope_only` for single-node
replica dedupe and `requires_global_vec_id` for cross-node dedupe.

The PG18 fixture builds a `source_identity = 'include'` SPIRE index with
`boundary_replica_count = 1` and two local stores, then verifies all four rows
are global and ready, with four primary assignments, four boundary replicas,
and two local-store placements.

This commit also folds in the optional packet `30774` review note by documenting
the Stage E rollup family and deterministic manifest freshness semantics in
`plan/design/spire-production-coordinator-executor.md`.

## Scope Boundaries

This is local index-level evidence over active placement files. It proves the
identity grouping and local multi-store boundary-replica shape, but it does not
claim remote multi-instance boundary-replica proof. That remains part of Stage E
fixture evidence.

## Validation

All commands were run on `f607c37fec14ec2c92d25006e884adf2805f5268`.

```text
cargo fmt --check
git diff --check -- src/lib.rs src/am/mod.rs src/am/ec_spire/mod.rs src/am/ec_spire/root/diagnostics.rs src/am/ec_spire/root/types.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md
cargo check --no-default-features --features pg18
cargo check --no-default-features --features "pg18 pg_test"
cargo pgrx test pg18 test_ec_spire_boundary_replica_identity_snapshot_global_ids
```

A final `git diff --check` including
`plan/design/spire-production-coordinator-executor.md` also passed after the
packet `30774` doc follow-up.

## Review Questions

- Is the assignment grouping by `vec_id` the right shape for checking primary
  and boundary-replica identity consistency?
- Are the `ready`, `local_scope_only`, and `requires_global_vec_id` categories
  strict enough for Stage E to consume without overclaiming remote proof?
- Does the test cover the important local multi-store boundary-replica case, or
  should this diagnostic also force a delta-row case before the Stage E fixture?
