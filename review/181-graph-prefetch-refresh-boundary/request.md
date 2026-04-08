# Review Request: Graph Prefetch Refresh Boundary

## Summary

- move graph-phase prefetched-result refresh behind one helper in `src/am/scan.rs`
- use the same graph cursor refresh path from both `amrescan` and post-emit graph advancement
- keep the graph hot path unchanged: `amgettuple` still only emits already-prefetched graph output

## What changed

- renamed the graph-phase prefill entry point to `refresh_graph_traversal_prefetch(...)`
- added `graph_traversal_prefetch_ready(...)` as the pure helper that:
  - reports whether prefetched graph output is ready
  - clears stale graph `current_result` state when duplicate drain is gone
- `tqhnsw_amrescan(...)` now enters graph execution through `refresh_graph_traversal_prefetch(...)`
- `advance_graph_traversal_after_emit(...)` now also uses `refresh_graph_traversal_prefetch(...)`
- updated the unit test around stale-current cleanup to target the new graph-prefetch-ready helper

## Why

- The previous slices already made the live graph tuple-production path consume only prefetched output.
- Graph cursor progression still had its stale-current cleanup and refresh work split across multiple call sites.
- This slice tightens the graph-first runtime boundary by making graph prefetch progression a single runtime helper shared by the two places that actually advance the graph cursor:
  - initial rescan setup
  - post-emit duplicate-drain completion

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether `refresh_graph_traversal_prefetch(...)` is the right graph-phase boundary for remaining cursor progression
- whether the new pure `graph_traversal_prefetch_ready(...)` helper keeps the unit-tested stale-current contract clear without pulling pg-dependent materialization into plain Rust tests
- whether this leaves the next useful A3 cut in the remaining shared result-state shell rather than further entry/fallback cleanup
