# Review Request: SPIRE Remote Node Capability Plan

- Code commit: `89c73ed0` (`Expose SPIRE remote node capability plan`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint exposes the pre-libpq node capability-check contract as a SQL
diagnostic:

- adds `SpireRemoteNodeCapabilityPlanRow`;
- adds `remote_node_capability_plan`;
- exports SQL function `ec_spire_remote_node_capability_plan(index_oid)`;
- derives capability rows from `ec_spire_remote_node_snapshot(...)`;
- reports the active epoch each node must serve and retain;
- reports required candidate format and extension version;
- reports conninfo source as `local` for node 0 and `remote_node_descriptor`
  for nonzero node IDs;
- reports remote identity, epoch-window, candidate-format, and extension-version
  check status columns;
- keeps local node 0 ready with no remote descriptor requirement;
- keeps nonzero nodes blocked as `requires_remote_node_descriptor` while the
  durable descriptor catalog is absent;
- updates the Phase 7 task note with the capability-plan surface;
- adds PG18 coverage for local and missing-descriptor remote capability plans.

This still does not store descriptors, expose raw conninfo, perform health
checks, open libpq connections, or execute remote SQL.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the row contract: required epoch window, candidate format, extension
   version, conninfo source, identity status, and readiness status.
2. Check that local node 0 remains `ready` without implying a remote descriptor.
3. Check that nonzero node IDs require a descriptor before any capability check
   can claim readiness.
4. Check that no raw connection strings or libpq execution behavior are exposed.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_node --no-default-features --features pg18`
  - Result: passed; 5 tests passed, including local and missing-descriptor
    capability-plan PG tests.
- `git diff --check`
