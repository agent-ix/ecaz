# Review Request: Graph Prefetch Materialization Boundary

## Summary

- collapse the graph refresh path onto a direct select-and-materialize boundary in `src/am/scan.rs`
- remove one more standalone graph helper layer from the live graph-first runtime path
- keep debug/test entry points intact while making graph prefetch materialization more local to refresh

## What changed

- `refresh_graph_traversal_prefetch(...)` now delegates straight to `prefetch_next_graph_traversal_result(...)`
- renamed the graph selection helper to `try_select_next_graph_traversal_result(...)`
- removed the old standalone `materialize_graph_traversal_result(...)` helper
- `prefetch_next_graph_traversal_result(...)` now owns:
  - graph-phase eligibility checks
  - the “no prefetched output already queued” guard
  - selection of the next graph traversal result
  - emitted-result marking
  - graph cursor materialization
- kept `materialize_next_bootstrap_frontier_result(...)` as a thin wrapper for debug/test surfaces

## Why

- After the previous A3 slices, graph traversal already owned readiness, emit, and refresh signaling through `GraphTraversalCursor`, but refresh still bounced through a separate “materialize next graph result” helper.
- This is the next bounded graph-side cut: the live graph prefetch path now owns the select-and-materialize step directly, which trims another layer from the scan shell without changing staged behavior.
- The remaining standalone surface is mostly there to preserve the existing debug/test hook contract, not because the runtime path still needs it.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the graph refresh path is now the right place for the select-and-materialize boundary
- whether the remaining thin `materialize_next_bootstrap_frontier_result(...)` wrapper should stay as a debug/test hook for now
- whether the next useful A3 cut is the remaining graph selection shell itself rather than more current-result plumbing
