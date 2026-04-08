# Review Request: Separate Fallback Result State

## Summary

- separate fallback result-state storage from the graph cursor result buffer in `src/am/scan.rs`
- keep graph on the existing `result_state` backing store
- route fallback emit/materialize/teardown through a dedicated `fallback_result_state`

## What changed

- added `fallback_result_state: ScanResultState` to `TqScanOpaque`
- `reset_scan_position(...)` and `mark_scan_exhausted(...)` now clear both graph and fallback result buffers
- `enter_linear_fallback_phase(...)` now resets fallback result state before fallback starts
- added `active_result_state_mut(...)` so phase-local pending-output emit uses:
  - graph storage in graph/exhausted phases
  - fallback storage in linear fallback
- `advance_linear_fallback_after_emit(...)` now tears down fallback state from `fallback_result_state`
- `materialize_linear_fallback_result(...)` now seeds fallback-only result storage instead of the graph buffer
- added focused unit coverage for:
  - active phase choosing fallback storage
  - fallback materialization staying out of graph result-state storage

## Why

- The previous slice introduced a dedicated graph cursor type but both phases still shared the same underlying result buffer.
- This is the next smallest real A3 cut: graph and fallback now have distinct result-state storage, so fallback no longer writes through the graph cursor’s buffer.
- That reduces another piece of shared scan-owned state and makes the remaining graph cursor ownership seam more concrete.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether separating fallback result storage is the right next A3 cut after the graph cursor seam
- whether any remaining graph/fallback result-state sharing should now be considered accidental rather than intentional
- whether the next useful step is making debug/runtime readers phase-aware instead of continuing to rely on graph-default result-state reads
