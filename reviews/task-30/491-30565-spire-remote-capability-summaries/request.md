# Review Request: SPIRE Remote Capability Summaries

- Code commit: `de928e85` (`Expose SPIRE remote capability summaries`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint batches two related pre-libpq readiness surfaces:

- adds `SpireRemoteNodeCapabilitySummaryRow`;
- adds `ec_spire_remote_node_capability_summary(index_oid)`;
- aggregates node capability-plan rows into one coordinator gate with
  ready/blocked node counts, local/remote node counts, missing descriptor
  counts, required candidate format, required extension version, status, and
  recommendation;
- adds `SpireRemoteEpochPublishReadinessRow`;
- adds `ec_spire_remote_epoch_publish_readiness(index_oid)`;
- exposes the remote-node descriptor gate for publishing distributed epoch
  placement metadata: remote node count, remote placement-state counts,
  blocked/missing descriptor counts, status, and recommendation;
- keeps local-only indexes ready with no remote descriptor requirement;
- keeps nonzero node IDs blocked as `requires_remote_node_descriptor` until the
  durable descriptor catalog lands;
- updates the Phase 7 task note with the two new readiness gates;
- adds PG18 coverage for local-only and missing-descriptor remote placements.

This still does not store descriptors, expose raw conninfo, perform health
checks, open libpq connections, execute remote SQL, or publish distributed
epochs.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that the capability summary accurately aggregates the per-node
   capability-plan contract without implying remote descriptors exist.
2. Check that the epoch publish readiness surface is appropriately conservative:
   any nonzero node remains blocked until descriptor metadata exists.
3. Check local-only behavior: status is `ready`, required candidate format stays
   `local`, and remote placement/node counts are zero.
4. Check that the surfaces remain diagnostic and do not expose raw connection
   strings or libpq execution behavior.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_node --no-default-features --features pg18`
  - Result: passed; 7 tests passed, including the two new capability-summary
    PG tests.
- `git diff --check`
