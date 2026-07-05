# Review Request: Phase-Aware Result-State Readers

## Summary

- make runtime/debug result-state readers phase-aware in `src/am/scan.rs` and `src/am/scan_debug.rs`
- stop assuming the graph cursor buffer is always the active result-state storage
- complete the previous fallback-storage split on the read side, not just the write side

## What changed

- added `active_result_state_ref(...)` alongside the existing mutable phase-aware accessor in `src/am/scan.rs`
- `active_result_state_ref(...)` now reads:
  - graph `result_state` during graph traversal and exhaustion
  - `fallback_result_state` during linear fallback
- updated scan debug/runtime readers to use `active_result_state_ref(...)` instead of reading `opaque.result_state` directly
- phase-aware readers now cover:
  - ordered-head and ordered-slot inspection
  - current-result state inspection
  - materialized candidate / pending-heap-tid debug surfaces
  - entry-candidate, lifecycle, neighbor, and heap-progress helpers

## Why

- The last A3 slice separated fallback result-state storage from the graph cursor buffer, but readers still implicitly treated the graph buffer as the only live one.
- This is the next smallest real A3 cut after the storage split: runtime/debug inspection now follows the active scan phase instead of reading stale or inactive graph state.
- That makes the graph/fallback storage split real for readers too, which reduces another leftover shared-state assumption in the scan shell.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether phase-aware read access is the right follow-through after separating fallback result-state storage
- whether any remaining direct `result_state` reads in runtime/debug surfaces are still intentional
- whether the next useful A3 cut is making more runtime readers phase-local through the cursor/accessor boundary instead of expanding shared scan-state plumbing
