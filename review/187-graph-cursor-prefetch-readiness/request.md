# Review Request: Graph Cursor Prefetch Readiness

## Summary

- move graph prefetch-readiness and post-emit refresh signaling behind `GraphTraversalCursor` in `src/am/scan.rs`
- remove more standalone graph-phase helper shell from the live scan path
- keep the graph-first staged behavior unchanged while making graph current-result progression more cursor-local

## What changed

- added `GraphTraversalCursor::needs_prefetch_refresh(...)`
- `refresh_graph_traversal_prefetch(...)` now uses the graph cursor directly for prefetched-output readiness
- removed standalone graph-phase helper functions for:
  - prefetched-output availability
  - prefetch-ready probing
  - post-emit refresh branching
- `produce_next_graph_traversal_heap_tid(...)` now delegates those decisions through `GraphTraversalCursor`
- updated focused unit coverage to exercise graph prefetch readiness through the cursor surface directly

## Why

- After the previous A3 slices, graph traversal already owned state and output emission through `GraphTraversalCursor`, but readiness/refresh decisions still lived in separate scan-owned helpers.
- This is the next bounded graph-side cut: the graph cursor now owns more of its own current-result lifecycle, not just storage and emit mechanics.
- That reduces more graph-specific runtime shell in `scan.rs` without widening the staged contract.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether graph prefetch readiness and post-emit refresh are now sitting on the right side of the graph cursor seam
- whether any remaining graph current-result lifecycle logic in `scan.rs` is still intentionally outside the cursor
- whether the next useful A3 cut is the remaining graph materialization/selection shell rather than more fallback-side work
