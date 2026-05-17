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

## Implementation

Completed work:

1. Added an in-place layer-neighbor iterator in `src/am/graph.rs`.
2. Kept `valid_neighbor_tids_for_layer(...)` as a materializing helper for
   tests/debug callers, but rewired it to build on the new iterator.
3. Rewrote the generic graph successor-loading helpers and the scan-local
   cached successor path in `src/am/scan.rs` so they iterate valid layer
   neighbors directly instead of first allocating a temporary
   `Vec<ItemPointer>`.
4. Preserved the existing graph/search semantics and test surfaces.

## Result

Current local read on the warm verified `10K`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell`, `cached-plan` seam:

```text
before: p50=10.932ms p95=13.137ms p99=15.059ms mean=10.993ms
after:  p50=10.753ms p95=12.784ms p99=14.034ms mean=10.720ms
```

This is still not a major C1-closing step, but it is a clearer win than the
prior scheduler cleanup: about a `2.5%` mean reduction on the current warm
steady-state seam, with p50 / p95 / p99 all moving in the right direction.

## Validation

- targeted graph/scan checks:
  - `cargo test valid_neighbor_tids_for_layer`
  - `cargo test layer0_successor_candidates_from_elements`
  - `cargo test run_layer0_beam_search_with_successors`
  - `cargo test current_candidate_frontier_head_tid`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`,
  `cached-plan` read recorded

All required gates were rerun and completed green after the final graph/scan
state.

## Conclusion

This slice is worth keeping. The result is large enough to say the per-layer
neighbor materialization churn was real, not just theoretical. The next slice
should keep pushing on the same theme and target the remaining temporary
successor/result vectors rather than opening another measurement seam.
