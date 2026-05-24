# Task 50 Review Request: HNSW Debug Guard-Owned Safe Helpers

## Summary

This slice makes two HNSW test/debug helpers safe at their exported test API
surface when they already open and own the required PostgreSQL relation or scan
guards:

- `debug_index_pages`
- `debug_gettuple_scan_heap_tids`

The remaining heap TID descriptor read stays inside
`ec_hnsw::scan_debug` as an explicit internal unsafe block. Test callers now
invoke these helpers directly, and HNSW test macros continue to wrap only the
debug helpers that still require unsafe.

## Files Changed

- `src/am/ec_hnsw/shared.rs`
- `src/am/ec_hnsw/scan_debug.rs`
- HNSW pg_test callers under `src/tests/`
- Shared HNSW test helpers in `src/tests/mod.rs`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2283 -> 2278`.
- Removed all HNSW test macro wrappers around:
  - `am::debug_index_pages(...)`
  - `am::debug_gettuple_scan_heap_tids(...)`
- Kept macro wrappers in place for other HNSW helpers that remain unsafe.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs src/am/ec_hnsw/shared.rs`
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
- `artifacts/hnsw-safe-helper-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
