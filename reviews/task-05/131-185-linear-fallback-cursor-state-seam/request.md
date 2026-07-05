# Review Request: Linear Fallback Cursor State Seam

## Summary

- introduce a dedicated `LinearFallbackCursor` in `src/am/scan.rs`
- move live fallback result-state progression behind that cursor instead of the generic active-state helper
- make graph and fallback tuple production use explicit phase-local result cursors

## What changed

- added `LinearFallbackCursor<'_>` over `fallback_result_state`
- the fallback cursor now owns:
  - result materialization into fallback storage
  - pending-output drain for fallback heap tids
  - post-emit teardown once duplicate drain is exhausted
- replaced `emit_pending_scan_heap_tid(...)` with fallback-specific pending-output emit
- removed the shared mutable active-result-state helper from the runtime path
- `produce_next_linear_fallback_heap_tid(...)` and `emit_materialized_linear_fallback_result(...)` now delegate through the fallback cursor
- updated focused unit coverage to exercise the fallback cursor directly

## Why

- Graph traversal already runs through an explicit `GraphTraversalCursor`, but fallback still used generic shared result-state plumbing.
- This is the next bounded A3 cut after phase-aware readers: both live phases now own their own result-state progression more explicitly.
- That reduces another piece of scan-owned generic runtime shell and makes the phase split more concrete without changing staged behavior.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether `LinearFallbackCursor` is the right symmetric boundary now that graph traversal already has its own cursor type
- whether any remaining shared result-state progression between graph and fallback is still intentional
- whether the next useful A3 cut is narrowing the remaining generic scan-owned current-result shell rather than adding more helper reshaping
