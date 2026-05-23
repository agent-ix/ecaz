# Task 50 Review Request: HNSW Rescan Gettuple Debug Safe Helpers

## Summary

This slice removes three remaining `unsafe fn` test/debug boundaries from HNSW gettuple rescan probes:

- `debug_gettuple_backward_after_rescan`
- `debug_gettuple_rescan_after_exhaustion`
- `debug_gettuple_rescan_after_partial`

The helpers already own their index relation guard, scan descriptor setup, rescan calls, and cleanup sequence. The only remaining raw heap TID read in the partial-rescan helper is now an explicit local unsafe block at the point of dereference, instead of requiring unsafe at the caller boundary.

The pg_test call sites in `src/tests/ec_hnsw_scan_gettuple.rs` now call these helpers directly and no longer route through `hnsw_scan_debug!`.

## Unsafe Burndown

- Previous broad count from packet 266: `2249`
- Current broad count: `2247`
- Net: `-2`

## Validation

Artifacts are under `reviews/task-50/267-hnsw-rescan-after-gettuple-debug-safe-helpers/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: passed; rustfmt emitted the existing stable-channel warnings for nightly-only import settings
- `hnsw-rescan-after-wrapper-grep.log`: no wrappers remain for these three helpers
- `unsafe-count.log`: `2247`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings

