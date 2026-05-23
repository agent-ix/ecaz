# Task 50 Review Request: HNSW Grouped Scan Debug Safe Helpers

## Summary

This slice makes grouped HNSW scan comparison/window debug helpers safe at their
exported test API surface:

- `debug_grouped_scan_comparison_rows`
- `debug_grouped_scan_comparison_summary`
- `debug_grouped_scan_order_drift_summary`
- `debug_grouped_scan_windowed_rows`
- `debug_grouped_scan_windowed_summary`

These helpers now compose the already-safe score-comparison scan helper and keep
the grouped-storage classifier behind its existing internal unsafe boundary.
HNSW pg_test and SQL-visible debug exports call the grouped helpers directly.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- HNSW grouped comparison pg_test callers under `src/tests/`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2272 -> 2264`.
- Removed all HNSW test macro wrappers around grouped comparison/window helpers.
- Removed unnecessary internal unsafe wrappers around calls to
  `debug_grouped_scan_comparison_rows`.

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
- `artifacts/hnsw-grouped-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
