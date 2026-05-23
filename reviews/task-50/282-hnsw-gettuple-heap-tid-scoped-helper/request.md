# Review Request: HNSW Gettuple Heap TID Scoped Helper

## Summary

This slice continues the HNSW debug unsafe burndown after packet 281.

Code commit: `fd60e9bc837733f0fda829bc1a01baf47afd39c5`

Changes:

- Added `debug_am_gettuple_heap_tid`, which calls the HNSW debug `gettuple` wrapper and snapshots `xs_heaptid` immediately only when the callback succeeds.
- Replaced repeated caller-local `unsafe { debug_scan_heap_tid(scan) }` and direct `(*scan).xs_heaptid` reads in scan-debug helpers with the new scoped helper.
- Converted `debug_rescan_candidate_frontier` from a direct `unsafe { debug_scan_opaque_mut(scan) }` borrow to the existing `debug_with_scan_opaque_mut` closure scope.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2156 -> 2148`.
- Removed direct raw heap-TID reads from:
  - `debug_gettuple_scan_heap_tids`
  - `debug_gettuple_scan_heap_tids_with_scores`
  - `debug_gettuple_scan_heap_tids_with_score_comparisons`
  - `debug_gettuple_exhaustion_state`
  - `debug_gettuple_rescan_after_exhaustion`
  - `debug_gettuple_rescan_after_partial`
- Remaining direct pattern hits are limited to wrapper internals:
  - `debug_with_scan_opaque_mut`
  - `debug_scan_heap_tid`
  - `debug_am_gettuple_heap_tid`

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass
- `cargo fmt --all -- --check`: exit 1 from existing repo-wide formatting drift; artifact retained for transparency, not treated as a slice regression

Artifact manifest: `reviews/task-50/282-hnsw-gettuple-heap-tid-scoped-helper/artifacts/manifest.md`

