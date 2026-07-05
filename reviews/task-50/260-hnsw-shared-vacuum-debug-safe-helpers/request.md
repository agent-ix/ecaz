# Task 50 Review Request: HNSW Shared Vacuum Debug Safe Helpers

## Summary

This slice makes HNSW shared metadata and vacuum debug helpers safe at their
exported test API surface when they own the required relation guards or callback
state:

- `debug_planner_tuning_snapshot`
- `debug_index_metadata`
- `debug_update_index_metadata`
- `debug_vacuum_stats`
- `debug_vacuum_remove_heap_tids`

The PostgreSQL metadata/vacuum operations remain inside explicit internal
unsafe blocks. HNSW pg_test call sites now invoke these helpers directly, and
the now-unused storage debug macro was removed.

## Files Changed

- `src/am/ec_hnsw/shared.rs`
- `src/am/ec_hnsw/vacuum.rs`
- HNSW pg_test callers under `src/tests/`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2276 -> 2272`.
- Removed all HNSW test macro wrappers around:
  - `am::debug_planner_tuning_snapshot(...)`
  - `am::debug_index_metadata(...)`
  - `am::debug_update_index_metadata(...)`
  - `am::debug_vacuum_stats(...)`
  - `am::debug_vacuum_remove_heap_tids(...)`
- Removed the unused `hnsw_storage_debug!` macro.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/shared.rs src/am/ec_hnsw/vacuum.rs`
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
- `artifacts/hnsw-shared-vacuum-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
