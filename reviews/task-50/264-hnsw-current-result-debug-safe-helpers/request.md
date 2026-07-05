# Task 50 Review Request: HNSW Current Result Debug Safe Helpers

## Summary

This slice makes the HNSW current-result and order-by scan debug helpers safe at
their exported test API surface:

- `debug_gettuple_exhaustion_state`
- `debug_gettuple_current_result_state`
- `debug_gettuple_orderby_score`
- `debug_gettuple_orderby_score_lifecycle`

The helpers own relation/scan setup and keep descriptor inspection behind
explicit internal unsafe boundaries. Callers in scan and recall tests now invoke
these helpers directly.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- `src/tests/ec_hnsw_scan_gettuple.rs`
- `src/tests/ec_hnsw_recall_helpers.rs`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2256 -> 2254`.
- Removed HNSW scan/recall debug macro wrappers around the newly-safe helpers.

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
- `artifacts/hnsw-current-result-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
