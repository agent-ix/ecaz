# Review Request: SPIRE Remote Node Model

- Code commit: `d6a7d106` (`Design SPIRE remote node model`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 multi-machine placement
- Agent: coder1

## Summary

This design checkpoint starts Phase 7 by defining the remote node model before
distributed libpq execution code lands:

- adds `plan/design/spire-remote-node-model.md`;
- reserves `node_id = 0` for the local coordinator node and treats nonzero
  node IDs as coordinator-scoped remote SPIRE storage nodes;
- separates stable node identity from connection strings, PostgreSQL OIDs,
  hostnames, and pod identities;
- defines the first remote node descriptor shape, including generation,
  remote index identity, state, served epoch window, extension version, and
  last error;
- defines `active`, `draining`, `disabled`, and `failed` read/write
  eligibility;
- keeps remote placements in the existing
  `pid -> node_id -> local_store_id -> object` placement map;
- defines remote placement interpretation for `local_store_id`, `store_relid`,
  object TID, object bytes, and one-primary-placement v1 semantics;
- defines stale-node conditions around served epoch, retained epoch window,
  remote index identity, extension/candidate-format compatibility, and node
  generation;
- records strict fail-closed and explicit degraded-skip behavior for remote
  reads before query execution exists;
- records remote candidate identity requirements for coordinator merge and row
  delivery;
- updates `plan/tasks/30-spire-ivf-foundation.md` to mark the Phase 7 remote
  node model checkpoint complete.

This does not implement libpq transport, remote search SQL, remote node
diagnostics, distributed epoch publication, or global `vec_id` rewrite.

## Files

- `plan/design/spire-remote-node-model.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that `node_id` scoping is correct: coordinator-index scoped, with
   `node_id = 0` local and nonzero IDs remote, and not coupled to DSNs, OIDs,
   hostnames, or deployment identities.
2. Check the node states and eligibility rules, especially whether `draining`
   should serve retained active placements while rejecting new placements.
3. Check the placement membership interpretation for remote entries:
   `local_store_id` and `store_relid` are remote-node concepts, while the
   coordinator keeps them diagnostic and routing metadata.
4. Check stale-node behavior against FR-041/FR-042: v1 treats stale placements
   as non-readable and uses `Skipped`/`Unavailable` degraded diagnostics rather
   than silently using stale candidate streams.
5. Check whether the design is explicit enough about the production dependency
   on global `vec_id` before durable cross-node merge claims.
6. Check the deferred list: replicated partition objects, automatic discovery,
   remote DDL, rebalancing, row fetch, libpq retry policy, and global vec-id
   rewrite should remain outside this first model checkpoint.

## Validation

- Documentation-only checkpoint.
- `git diff --check`
