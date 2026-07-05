# Review Request: Cursor-Owned Output Emission

## Summary

- move live tuple emission behind the graph and fallback cursor types in `src/am/scan.rs`
- reduce more scan-owned emit/materialize shell around the graph-first and fallback phases
- keep staged scan behavior unchanged while making output progression more phase-local

## What changed

- added `GraphTraversalCursor::emit_prefetched_output(...)`
- added `LinearFallbackCursor::emit_pending_output(...)`
- added `LinearFallbackCursor::emit_materialized_output(...)`
- removed the standalone fallback pending-output emit helper
- removed the standalone fallback materialize-and-emit helper
- removed the standalone graph emit helper
- `produce_next_graph_traversal_heap_tid(...)` now emits through the graph cursor directly
- `produce_next_linear_fallback_heap_tid(...)` now emits through the fallback cursor directly for both:
  - already-pending duplicate drain
  - newly materialized fallback results

## Why

- After the previous A3 slices, graph and fallback already had distinct result-state storage, readers, and cursor types.
- The remaining gap was that scan still owned too much of the actual emit/materialize shell around those cursors.
- This is the next bounded A3 cut: both live phases now own more of their output progression directly, which shrinks the generic scan runtime shell without changing the staged contract.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether moving output emission into the cursor types is the right next A3 seam after the state split
- whether any remaining phase-local emit/materialize logic in `scan.rs` should now be considered incidental shell rather than a stable boundary
- whether the next useful cut is the remaining graph current-result refresh shell rather than more fallback-side cleanup
