# Review Request: Graph Cursor State Seam

## Summary

- introduce a dedicated `GraphTraversalCursor` type in `src/am/scan.rs`
- move live graph result-state mechanics behind that cursor while keeping storage and staged behavior unchanged
- keep fallback on the old `ScanResultState` path for now

## What changed

- added `GraphTraversalCursor<'_>` over the existing graph `ScanResultState`
- the graph cursor now owns:
  - prefetched-output detection
  - stale-current cleanup for graph prefetch readiness
  - graph result materialization into pending duplicate drain
  - taking prefetched graph output for emit
- `graph_traversal_prefetch_ready(...)` now delegates to the graph cursor
- `materialize_graph_traversal_result(...)` now uses the graph cursor instead of the shared seed helper
- `emit_prefetched_graph_traversal_result(...)` now drains pending graph output through the cursor
- added focused unit coverage for graph cursor pending-output drain

## Why

- The review batch for 171–181 explicitly said the next useful A3 direction is graph result-state ownership behind a dedicated cursor struct.
- This slice starts that shift without widening the storage change: graph traversal now has a dedicated runtime type for its ordered-result state mechanics, even though the underlying buffer still lives in `TqScanOpaque`.
- That reduces more graph-phase dependence on the generic shared result-state shell and leaves fallback untouched.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether `GraphTraversalCursor` is the right next A3 boundary for graph result-state ownership
- whether the cursor now owns the right graph-phase mechanics without overreaching into fallback
- whether this leaves the next useful cut as storage-level separation between graph and fallback result state, rather than more helper churn
