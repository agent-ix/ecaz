# Review Request: Direct Graph Prefetch Materialization

## Summary

- materialize graph results directly inside the search-owned `select_next_with_refill(...)` path in `src/am/scan.rs`
- remove the live graph lane’s remaining `SelectedScanResult` selection helper hop
- keep staged behavior unchanged while shrinking one more graph-only shell out of `scan.rs`

## What changed

- removed `select_scan_candidate_result(...)`
- removed `try_select_next_graph_traversal_result(...)`
- `prefetch_next_graph_traversal_result(...)` now:
  - loads candidate elements directly inside the selection closure
  - rejects deleted / empty-heaptid elements in-place
  - marks emitted graph elements in-place
  - materializes directly into `GraphTraversalCursor`
  - returns success based on whether graph prefetch materialization actually happened

## Why

- After the previous A3 slices, the live graph path already owned readiness, emit, refresh, and the graph-prefetch boundary itself.
- The remaining live shell was that graph prefetch still selected into an intermediate `SelectedScanResult` helper path before materializing.
- This is the next bounded runtime cut: the graph lane now materializes directly during search-owned frontier selection, which removes one more scan-owned intermediate from the live graph path.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether direct graph materialization inside the selection closure is the right next A3 boundary
- whether any remaining graph-only shell around `select_next_with_refill(...)` is still justified
- whether the next useful cut is now search/frontier-side rather than more scan-local graph cursor cleanup
