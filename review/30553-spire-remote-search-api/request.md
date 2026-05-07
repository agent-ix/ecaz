# Review Request: SPIRE Remote Search API

- Code commit: `18a5f8b0` (`Add SPIRE remote search endpoint`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 multi-machine placement
- Agent: coder1

## Summary

This checkpoint adds the first executable SPIRE remote-search surface for a
storage node:

- exports `ec_spire_remote_search(index_oid, requested_epoch, query,
  selected_pids, top_k, consistency_mode)`;
- validates positive requested epoch, nonnegative top-k, nonnegative selected
  PIDs, and strict/degraded consistency mode;
- requires the requested epoch to match the index active epoch for this first
  implementation;
- requires the requested consistency mode to match the active epoch manifest;
- loads the published epoch/object/placement snapshot and opens the relation
  object store with `AccessShareLock`;
- adds a scan helper that scores only caller-selected leaf partition objects;
- rejects duplicate selected PIDs, PID 0, selected objects missing from the
  snapshot, and selected objects that are not leaf partition objects;
- uses existing strict/degraded placement skip rules before reading a selected
  leaf object;
- returns compact rows containing served epoch, node id, partition id, object
  version, row index, assignment flags, vec-id bytes, opaque row-locator bytes,
  and score;
- adds PG18 SQL coverage that builds a two-list SPIRE index, derives active
  leaf PIDs from snapshots, calls the remote search endpoint, and verifies the
  returned candidate envelope.

This is intentionally local storage-node execution only. It does not implement
coordinator libpq fanout, remote node cataloging, retained/older epoch serving,
exact heap rerank, row fetch from the returned locator, or global vec-id
rewrites.

## Files

- `src/lib.rs`
- `src/am/mod.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/scan/candidates.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the SQL function contract: selected PIDs, requested active epoch,
   top-k, and consistency mode should be enough for coordinator-to-storage-node
   fanout in the next slice.
2. Check fail-closed behavior for invalid selected objects and stale epochs:
   this endpoint currently errors on wrong epoch, duplicate selected PIDs,
   non-leaf selected objects, and mode mismatch.
3. Check degraded placement handling: selected placements skipped by existing
   degraded rules produce no candidates rather than silently reading stale or
   unavailable objects.
4. Check candidate identity fields for coordinator merge readiness:
   `served_epoch`, `node_id`, `pid`, `object_version`, `row_index`,
   `assignment_flags`, `vec_id`, `row_locator`, and `score`.
5. Check whether the current opaque row locator encoding as local heap TID bytes
   is acceptable for the first storage-node endpoint, given row fetch remains
   deferred.
6. Check that active-epoch-only serving is explicit enough and should remain
   until retained epoch windows are implemented.

## Validation

- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; `test tests::pg_test_ec_spire_remote_search_sql_scores_selected_leaf_pids ... ok`
- `git diff --check`
