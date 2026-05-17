# Review Request: SPIRE Borrowed Top-Graph Routing

Phase 9.3 removes the per-query adjacency copy in top-graph routing and drops
the extra full query-to-centroid pass used only to create a non-negative
distance offset.

Code checkpoint: `07866db3` (`Use borrowed top graph routing view`)

## Scope

- Adds `VamanaGraphView` and `greedy_search_view` so greedy search can operate
  over borrowed adjacency without requiring `Vec<Vec<u32>>` materialization.
- Keeps the existing `greedy_search(&VamanaGraph, ...)` API as a delegating
  wrapper for build and existing tests.
- Routes SPIRE top graphs through a `SpireTopGraphGreedyView` over the loaded
  durable graph or build draft.
- Uses monotonic `-inner_product(query, centroid)` as the query-time distance
  instead of scanning every root child once to compute a constant IP offset.
- Marks Phase 9.3 complete in the detailed and summary Task 30 files.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 route_top_graph --lib`
- `cargo test --no-default-features --features pg18 greedy_search --lib`
- `cargo test --no-default-features --features pg18 top_graph_object_routes_recursive_children_to_leaf_routes --lib`

## Review Focus

- Confirm that the borrowed `VamanaGraphView` trait is narrow enough for both
  top-graph routing and existing Vamana build/search callers.
- Confirm that replacing `offset - inner_product` with `-inner_product` is
  semantically equivalent for query-time ordering and tie-break behavior.
- Check whether future Phase 10 scan caching should build on this view instead
  of adding a separate top-graph-specific cache API.
