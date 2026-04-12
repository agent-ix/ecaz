# Review Request: C1 Layer Neighbor Iteration Without Temp Vec

## Context

Packet `269` removed the eager `BeamSearch::forget_queued` heap drain/rebuild
path and produced only a small warm improvement:

- before: `p50=11.028ms`, `mean=11.041ms`
- after: `p50=10.932ms`, `mean=10.993ms`

That was worth keeping, but it also confirmed that the remaining C1 gap will
not close by polishing tiny scheduler seams one at a time.

The next obvious warm-path allocation seam is in the graph/scan neighbor walk:
the code frequently materializes a `Vec<ItemPointer>` of valid neighbors for a
layer, then immediately iterates that `Vec` to load elements and score them.

## Problem

In the current graph and scan helpers:

- `graph::valid_neighbor_tids_for_layer(...)` allocates a new `Vec`
- callers like `load_layer0_successor_candidates`,
  `search_layer_result_candidates`, and the scan-local cached successor path
  immediately consume that `Vec` once
- the hot scan path already has the decoded adjacency in hand, so this extra
  temporary allocation is avoidable

This is not expected to be a huge win by itself, but it is a direct
per-expansion allocation cut in the warm graph traversal path.

## Planned work

1. Add an in-place layer-neighbor iteration helper in `src/am/graph.rs`.
2. Rewrite the hot successor-loading paths to use that helper directly instead
   of allocating an intermediate `Vec<ItemPointer>`.
3. Preserve the existing public helper behavior for tests/debug callers that
   still want a materialized `Vec`.
4. Rerun the full checkpoint gate and the warm verified `10K`, `m=8`,
   `ef_search=40`, `warm-after-prime3`, `per-cell`, `cached-plan` seam.

## Exit criteria

- hot successor-loading paths stop materializing a temporary neighbor-tid `Vec`
- existing graph/search tests remain green
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`,
  `cached-plan` read recorded
