# Review Request: Graph Phase Emit Boundary

## Summary

- give the graph-first scan phase its own prefetched emit-and-advance helper in `src/am/scan.rs`
- keep the graph phase on an explicit phase-local tuple-production boundary, matching the fallback side more closely
- update one stale entry-candidate lifecycle contract so immediate graph exhaustion after the first tuple drain is represented explicitly

## What changed

- added `emit_prefetched_graph_traversal_result(...)`
- `produce_next_graph_traversal_heap_tid(...)` now:
  - ensures the graph phase has prefetched output
  - delegates the actual emit-and-advance step to the new graph-phase helper
- updated `debug_entry_candidate_lifecycle(...)` to report whether the graph lane is already exhausted immediately after the first tuple drain
- updated the pg assertion in `src/lib.rs` so partial graph progress may now mean:
  - a remaining frontier candidate
  - a concrete current result
  - or an already-exhausted graph lane

## Why

- Recent A3 slices already made fallback tuple production explicitly phase-local.
- The graph phase was still doing its emit-and-advance sequence inline inside `produce_next_graph_traversal_heap_tid(...)`.
- This slice makes the live graph cursor boundary more explicit without changing the staged graph-first contract.
- The pg lifecycle note avoids treating immediate post-drain exhaustion as a failure when the graph lane has legitimately finished.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the graph-phase emit-and-advance helper is the right boundary for the remaining A3 cursor shell
- whether the updated entry-candidate lifecycle contract reflects the staged graph-first runtime accurately
- whether this leaves the next useful A3 cut in graph result-state ownership rather than more entry/fallback gating
