# Review Request: SPIRE Remote Node Snapshot

- Code commit: `b762edb1` (`Expose SPIRE remote node snapshot`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint adds a SQL-visible node diagnostic surface before libpq fanout
execution or durable remote-node descriptors land:

- adds `SpireRemoteNodeSnapshotRow`;
- adds `remote_node_snapshot`;
- exports SQL function `ec_spire_remote_node_snapshot(index_oid)`;
- derives rows from the active epoch placement directory;
- emits one row per node ID present in active placements;
- reports local node readiness with `node_kind = 'local'`,
  `descriptor_state = 'active'`, and `status = 'ready'`;
- reports nonzero node IDs as `node_kind = 'remote'`,
  `descriptor_state = 'missing'`, and
  `status = 'requires_remote_node_descriptor'`;
- includes descriptor-generation, placement counts, placement-state counts,
  local-store count, served/retained epoch placeholders, extension version,
  last error, status, and recommendation fields;
- uses the coordinator fanout manifest loader so remote placement diagnostics
  remain visible instead of failing through local-store validation;
- adds PG18 coverage for local-only node readiness and placeholder remote-node
  missing-descriptor diagnostics;
- updates the Phase 7 task note with the new diagnostic.

This is intentionally diagnostic. It does not add a durable node descriptor
catalog, raw conninfo storage, node health checks, or libpq execution.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that local `node_id = 0` reports as ready without implying a remote
   descriptor exists.
2. Check that nonzero node IDs remain visible as remote placement diagnostics
   and fail readiness through `requires_remote_node_descriptor`.
3. Check that the surface does not expose conninfo or imply libpq transport has
   landed.
4. Check that the coordinator fanout manifest loader is the right boundary for
   this diagnostic, since the regular local-store loader rejects remote nodes.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_node --no-default-features --features pg18`
  - Result: passed; 3 tests passed, including local and missing-descriptor
    remote-node snapshot PG tests.
- `git diff --check`
