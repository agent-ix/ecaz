# Review Request: Phase-Local Fallback Output Progression

## Summary

- make the linear fallback path own its post-emit teardown behavior in `src/am/scan.rs`
- clear stale `current_result` state immediately after the last fallback duplicate drains
- keep graph traversal and linear fallback on distinct post-emit progression contracts

## What changed

- added `advance_linear_fallback_after_emit(...)`
- `produce_next_linear_fallback_heap_tid(...)` now:
  - drains any already-pending fallback output
  - clears `current_result` when that duplicate drain finishes
  - still materializes on demand when no fallback output is pending
- added unit coverage for:
  - keeping the fallback current result while duplicate drain remains
  - clearing the fallback current result after the last duplicate drains

## Why

- The previous A3 slice made graph traversal behave like a prefetched cursor.
- Linear fallback was still leaving stale `current_result` state behind after its last duplicate emit.
- This slice makes phase-local output progression explicit:
  - graph traversal: emit, then eagerly advance/prefill
  - linear fallback: emit, then tear down stale current-result state when the fallback result is fully drained

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the fallback-specific teardown belongs in `src/am/scan.rs` at this phase boundary
- whether clearing stale fallback `current_result` immediately after the last duplicate drain is the right runtime contract
- whether this makes the graph-vs-fallback result-state behavior more coherent without changing fallback tuple production semantics
