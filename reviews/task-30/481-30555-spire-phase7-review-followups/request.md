# Review Request: SPIRE Phase 7 Review Followups

- Code commit: `510749b6` (`Address SPIRE phase 7 review feedback`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 5/6/7 feedback followups
- Agent: coder1

## Summary

This checkpoint addresses the reviewer feedback left on the Phase 7 packets and
two cross-cutting Phase 5/6 notes that should be fixed before libpq fanout:

- documents the production-blocking global `vec_id` precondition for multi-node
  candidate merge in `plan/design/spire-remote-node-model.md`;
- documents active-epoch-only fanout until retained-epoch serving exists;
- adds code comments that remote `row_locator` bytes are opaque to coordinators
  and that `top_k = 0` is a valid empty endpoint probe;
- changes `ec_spire_remote_search.assignment_flags` from `integer` to
  `smallint` to match the internal flag width;
- documents that the storage-node endpoint scores coordinator-selected leaves
  and never runs top-graph or recursive routing itself;
- adds a `merge_remote_search_candidates` doc-comment warning that inputs must
  share one coordinator-scoped `vec_id` namespace until global vec-id lands;
- removes the dead `vec_id` tie-break inside same-vec-id merge groups;
- dedupes insert-routed leaf PIDs before splitting primary vs. boundary
  replicas, mirroring the build-path defensive behavior;
- records that insert routing intentionally remains recursive/non-top-graph for
  v1;
- replaces top-graph build's O(n^2) max-centroid-IP offset with one
  `max_centroid_norm_sq` pass;
- documents the top-graph Vamana distance as an IP-derived non-metric
  pseudo-distance.

## Files

- `plan/design/spire-remote-node-model.md`
- `plan/design/spire-top-level-graph.md`
- `src/am/ec_spire/build/top_graph.rs`
- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/scan/candidates.rs`
- `src/lib.rs`

## Review Focus

1. Check that the global `vec_id` warning is explicit enough to prevent
   accidental multi-node use before global vec-id encoding lands.
2. Check that `assignment_flags smallint` is the right SQL surface for the
   current `u16` flag set.
3. Check the insert-route dedupe: it preserves route order, then chooses the
   first unique leaf as primary and later unique leaves as replicas.
4. Check the top-graph offset change from pairwise max IP to max centroid norm
   squared; it should preserve nonnegative pseudo-distance while removing the
   O(n^2) setup pass.
5. Check that comments document, without overpromising, the intentional
   top-graph bypass in storage-node remote search and post-build insert.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; 1 test passed.
- `cargo test --lib remote_candidate_merge --no-default-features --features pg18`
  - Result: passed; 3 tests passed.
- `cargo test --lib top_graph_build --no-default-features --features pg18`
  - Result: passed; 6 tests passed.
- `cargo test --lib boundary_replica_build_writes_and_dedupes_scan --no-default-features --features pg18`
  - Result: passed; 1 test passed.
- `git diff --check`
