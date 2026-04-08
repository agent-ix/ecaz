# Review Request: Graph Prefetched Hot Path

## Summary

- make graph-phase tuple production in `src/am/scan.rs` consume only already-prefetched output
- keep first graph materialization in `amrescan` and post-emit graph advancement, instead of re-running readiness work in the `amgettuple` hot path
- update one stale raw-frontier pg test in `src/lib.rs` so an already-prefetched ordered head no longer looks like a failed manual frontier consume

## What changed

- added `graph_traversal_has_prefetched_output(...)`
- `prefill_graph_traversal_result(...)` now reuses that helper when deciding whether graph output is already ready
- `produce_next_graph_traversal_heap_tid(...)` now:
  - requires prefetched graph output
  - emits through the existing graph-phase helper
  - no longer calls graph prefill/materialization from inside the tuple-production hot path
- added a unit test covering the prefetched-output contract
- updated `test_tqhnsw_frontier_head_refills_from_consumed_neighbors()` so an empty raw frontier is accepted when `amrescan` has already materialized the ordered head into current-result state

## Why

- Recent A3 slices already moved graph prefill to `amrescan` and post-emit advancement.
- The graph tuple-production path was still rechecking readiness as if it might need to materialize on demand.
- This slice tightens the live graph-first contract: once seeded, graph traversal behaves more like a prefetched ordered cursor and less like a lazily materialized selector inside `amgettuple`.
- The pg adjustment keeps a test-only raw-frontier helper aligned with that staged contract.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the graph-phase hot path should now rely only on prefetched output
- whether the raw-frontier pg helper is correctly treated as optional once the ordered head has already been materialized into current-result state
- whether this leaves the next useful A3 cut in remaining graph result-state ownership rather than more fallback or bootstrap cleanup
