# Task 50 Review Request: HNSW Scan Lifecycle Debug Safe Helpers

## Summary

This slice makes the HNSW scan lifecycle and rescan error-path debug helpers
safe at their exported test API surface:

- `debug_begin_end_scan`
- `debug_end_scan_twice`
- `debug_rescan_query_dimensions`
- `debug_rescan_overwrites_query_dimensions`
- `debug_rescan_null_query`
- `debug_rescan_with_index_qual`
- `debug_rescan_with_unused_key_buffer`
- `debug_rescan_with_multiple_orderbys`
- `debug_gettuple_without_rescan`
- `debug_gettuple_after_rescan`
- `debug_gettuple_after_rescan_result`

The helpers own their relation guard and scan descriptor setup. Raw scan opaque
queries remain explicit internal unsafe boundaries, and prepared-query pointer
length inspection is centralized behind `debug_prepared_query_lengths`.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- `src/tests/ec_hnsw_scan_gettuple.rs`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2261 -> 2256`.
- Removed HNSW scan debug macro wrappers around lifecycle/rescan helpers.
- Centralized repeated prepared-query raw pointer inspection in one internal
  helper.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`

Known pre-existing warnings are unchanged:

- normal `cargo check`: SPIRE DML test re-export unused-import warning in
  `src/am/mod.rs`
- `pg_test` no-run: Hadamard test-only helper dead-code warnings

## Artifacts

- `artifacts/manifest.md`
- `artifacts/unsafe-count.log`
- `artifacts/hnsw-lifecycle-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
