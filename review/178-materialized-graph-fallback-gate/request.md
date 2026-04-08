# Review Request: Materialized Graph Fallback Gate

## Summary

- make `amrescan` choose graph-vs-fallback based on whether graph traversal can materialize the first ordered result
- stop using the weaker “graph has a seeded candidate head” gate
- keep the linear path only when the graph lane cannot produce the first ordered result up front

## What changed

- `tqhnsw_amrescan(...)` now calls `prefill_graph_traversal_result(...)` directly
- if graph traversal cannot materialize the first ordered result during `amrescan`, the scan enters explicit linear fallback immediately
- removed the old `graph_traversal_has_seeded_candidates(...)` helper and its unit coverage because the live gate is now first-result materialization itself

## Why

- The prior A3 slice already moved fallback entry off a separate seeded boolean and onto live graph state.
- This slice tightens that one step further:
  - before: graph lane won if a frontier head existed
  - now: graph lane wins only if it can actually materialize the first ordered result
- That reduces another staged bootstrap-era shell assumption and makes the live graph-first path more honest about when fallback is still needed.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether first-result materialization is the right `amrescan` boundary for graph-vs-fallback selection
- whether this improves the A3 runtime contract without overreaching into planner or ordered-result buffering changes
- whether the fallback shell is now narrow enough that the next useful cut should move deeper into graph result-state ownership rather than more entry gating
