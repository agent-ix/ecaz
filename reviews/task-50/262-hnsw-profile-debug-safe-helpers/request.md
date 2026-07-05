# Task 50 Review Request: HNSW Profile Debug Safe Helpers

## Summary

This slice makes HNSW profiling/debug helpers safe at their exported test API
surface:

- `debug_profile_ordered_scan`
- `debug_profile_ordered_scan_with_limit`
- `debug_profile_ordered_scan_with_heap_fetch`
- `debug_grouped_rerank_profile`
- `debug_turboquant_scan_stage_profile`

The helpers already own the relation, snapshot, slot, or scan guards they need.
The remaining raw scan opaque read and scan guard construction are explicit
internal unsafe boundaries. HNSW pg_test and SQL-visible debug export callers no
longer route these helpers through unsafe debug macros, and now-unused runtime
debug macros were removed.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- HNSW profile/runtime pg_test callers under `src/tests/`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2264 -> 2261`.
- Removed HNSW test macro wrappers around profile/rerank/stage debug helpers.
- Removed unused `hnsw_runtime_debug!` macros from runtime test modules.

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
- `artifacts/hnsw-profile-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
