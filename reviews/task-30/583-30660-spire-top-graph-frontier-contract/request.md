# Review Request: SPIRE Top-Graph Frontier Contract

## Summary

Phase 9.1 now has a durable top-graph frontier contract and the existing
top-graph snapshot exposes the terms needed to distinguish root fanout, graph
node count, routing levels, and active leaf count.

Code checkpoint: `4b223d4c` (`Define SPIRE top graph frontier contract`)

## Scope

- Adds ADR-054:
  `spec/adr/ADR-054-spire-top-graph-frontier-contract.md`.
- Records that SPIRE top-graph nodes are the active root/top routing object's
  child frontier.
- Clarifies that the future scale build must make that root/top child set
  large enough for graph routing instead of always compressing it down to
  `recursive_fanout`.
- Updates `ec_spire_index_top_graph_snapshot(...)` with frontier diagnostics:
  - `frontier_kind`;
  - `frontier_parent_level`;
  - `frontier_child_level`;
  - `frontier_node_count`;
  - `root_child_count`;
  - `active_leaf_count`.
- Makes the top-graph snapshot report mismatch statuses for graph/root
  inconsistencies:
  - `missing_root`;
  - `root_mismatch`;
  - `level_mismatch`;
  - `frontier_mismatch`.
- Marks Phase 9.1 complete in the Task 30 phase task file and the main Task 30
  overview.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 top_graph --lib`
  - 32 top-graph Rust tests passed.
  - `tests::pg_test_ec_spire_top_graph_snapshot_sql` passed after pgrx SQL
    generation/install.

## Notes

This checkpoint does not remove the single-tuple top-graph storage ceiling and
does not change the recursive build stop condition. Those are the next Phase 9
slices: scalable top-graph storage and build-time frontier sizing.
