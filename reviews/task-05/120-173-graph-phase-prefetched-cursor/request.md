# Review Request: Graph Phase Prefetched Cursor

## Summary

- make the graph-traversal scan phase drain only already-prefetched outputs in `src/am/scan.rs`
- keep graph-phase advancement eager by prefilling the next graph result immediately after each emit
- leave linear fallback on its own on-demand materialization path
- align pg/debug surfaces with the stricter graph-first prefetched-cursor contract

## What changed

- `produce_next_scan_heap_tid(...)` no longer drains pending output through one shared pre-phase shell
- `produce_next_graph_traversal_heap_tid(...)` now treats graph traversal as a prefetched cursor:
  - emit one pending heap TID
  - advance/prefill the next graph result
  - return false if graph traversal has already exhausted instead of lazily materializing inside `amgettuple`
- `produce_next_linear_fallback_heap_tid(...)` now owns its own pending-drain behavior before linear materialization
- debug helpers and pg tests now prefer `current_result` when graph-prefetch has already materialized the next ordered result, and they stop assuming the raw visible frontier always retains full width or direct entry-neighbor provenance after prefill/top-up

## Why

- This is the next A3 runtime step after making graph traversal the primary ordered lane.
- Seeded graph scans now behave more like a dedicated prefetched cursor and less like a shared scan shell that lazily rematerializes results inside `amgettuple`.
- The runtime boundary is cleaner:
  - graph phase: drain prefetched graph outputs and eagerly advance
  - fallback phase: scan/materialize on demand

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the graph-phase runtime boundary in `src/am/scan.rs` is now coherent: prefetched graph cursor vs on-demand linear fallback
- whether the eager post-emit graph advancement is the right A3 state-machine step
- whether the updated debug/pg-test contracts match the intended graph-first staged behavior without overfitting to transient raw-frontier details
