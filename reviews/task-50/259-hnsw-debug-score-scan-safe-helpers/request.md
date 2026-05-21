# Task 50 Review Request: HNSW Score Scan Debug Safe Helpers

## Summary

This slice makes the HNSW gettuple debug helpers that return scan scores safe
at their exported test API surface:

- `debug_gettuple_scan_heap_tids_with_scores`
- `debug_gettuple_scan_heap_tids_with_score_comparisons`

Both helpers open the heap-backed scan state, rescan it with the supplied query,
materialize debug rows, and consume the scan state before returning. The live
scan descriptor operations now sit inside one bounded internal unsafe region per
helper. Test call sites no longer route these helpers through HNSW unsafe debug
macros.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- HNSW pg_test callers under `src/tests/`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2278 -> 2276`.
- Removed all HNSW test macro wrappers around:
  - `am::debug_gettuple_scan_heap_tids_with_scores(...)`
  - `am::debug_gettuple_scan_heap_tids_with_score_comparisons(...)`
- Removed now-unnecessary internal unsafe wrappers in grouped scan comparison
  helpers after the score-comparison helper became safe.

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
- `artifacts/hnsw-score-helper-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
