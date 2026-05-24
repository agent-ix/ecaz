# Task 50 Review Request: HNSW Oracle Debug Safe Helpers

## Summary

This slice converts the remaining HNSW oracle debug APIs in `src/am/ec_hnsw/scan_debug.rs` from unsafe public helpers to safe test/debug helpers:

- `debug_top_level_oracle_scan_heap_tids`
- `debug_top_level_oracle_k_seed_heap_tids`
- `debug_top_level_oracle_k_seed_scan_heap_tids`
- `debug_layer_oracle_k_carrydown_scan_heap_tids`
- `debug_layer_oracle_k_seed_layer0_neighbor_heap_tids`
- `debug_exact_seed_scan_heap_tids`

The shared raw-pointer pattern for cached quantizer and prepared query access is now centralized in `debug_with_oracle_score_parts`, which scopes those references to a closure. The recall tests call these helpers directly, and the now-unused `hnsw_recall_debug!` unsafe wrapper macro was removed.

After this packet, `src/am/ec_hnsw/scan_debug.rs` has no `pub(crate) unsafe fn debug_*` entries remaining.

## Unsafe Burndown

- Previous broad count from packet 268: `2237`
- Current broad count: `2221`
- Net: `-16`

## Validation

Artifacts are under `reviews/task-50/269-hnsw-oracle-debug-safe-helpers/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: passed; rustfmt emitted the existing stable-channel warnings for nightly-only import settings
- `hnsw-unsafe-debug-fn-grep.log`: no `pub(crate) unsafe fn debug_*` entries remain in HNSW `scan_debug.rs`
- `unsafe-count.log`: `2221`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings

