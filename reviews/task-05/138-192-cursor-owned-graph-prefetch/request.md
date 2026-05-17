# Review Request: Cursor-Owned Graph Prefetch

## Summary

- move the live graph prefetch lifecycle behind `GraphTraversalCursor` in `src/am/scan.rs`
- remove the free `refresh_graph_traversal_prefetch(...)` helper from the runtime shell
- keep the debug/test prefetch entry point as a thin wrapper while making the graph cursor own readiness, prefetch, and exhaustion handling

## What changed

- added `GraphTraversalCursor::prefetch_next(...)`
- added `GraphTraversalCursor::ensure_prefetched_output(...)`
- `amrescan` now asks the graph cursor to ensure the first prefetched ordered result instead of calling a free refresh helper
- graph tuple production now asks the graph cursor to refresh itself after the last duplicate drains
- extended `GraphTraversalPrefetchContext` to carry the graph result-state pointer directly, so graph candidate materialization no longer re-enters `graph_traversal_cursor(...)` mid-selection
- removed `refresh_graph_traversal_prefetch(...)`
- kept `prefetch_next_graph_traversal_result(...)` only as a thin debug/test wrapper over the cursor-owned prefetch path

## Why

- The previous slice packaged the raw frontier-selection operations behind a narrow context, following the batch review guidance.
- The next remaining live shell was that scan runtime still reached a free graph refresh/prefetch helper pair instead of a cursor-owned boundary.
- This is the next bounded A3 cut: graph traversal now owns more of its own ordered-result readiness and refresh lifecycle, while fallback semantics stay unchanged and exhaustion still terminates the graph lane rather than falling back.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether `GraphTraversalCursor` is now the right live boundary for graph readiness/prefetch/exhaustion progression
- whether the remaining thin `prefetch_next_graph_traversal_result(...)` wrapper is acceptable as debug/test-only surface
- whether the remaining A3 shell is now small enough to close after one more ownership cut, or if more frontier movement is still needed
