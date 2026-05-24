# Task 50 Review Request: Index Relation Heap OID Guard

## Summary

This slice adds a `pg_test`/test-only `IndexRelationGuard::heap_relation_oid()`
accessor and uses it in debug/test paths that already own an
`IndexRelationGuard`.

The intent is to keep heap-OID lookup tied to the RAII guard that proves the
index `Relation` pointer is live, rather than repeating caller-side raw
`Relation` unsafe blocks.

## Files Changed

- `src/storage/relation_guard.rs`
- `src/am/ec_hnsw/scan_debug.rs`
- `src/am/ec_hnsw/vacuum.rs`
- `src/am/ec_diskann/routine.rs`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2405 -> 2402`.
- Touched direct unsafe blocks: `239 -> 236`.
- Removed four caller-side heap-OID lookup unsafe blocks.
- Added one guard-owned heap-OID unsafe block behind
  `#[cfg(any(test, feature = "pg_test"))]`.

## Validation

- `rustfmt --edition 2021 --check src/storage/relation_guard.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_diskann/routine.rs src/am/ec_hnsw/vacuum.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`

Known pre-existing warnings are unchanged:

- normal `cargo check`: SPIRE DML test re-export unused-import warning in
  `src/am/mod.rs`
- `pg_test` no-run: Hadamard test-only helper dead-code warnings

## Artifacts

- `artifacts/manifest.md`
- `artifacts/unsafe-counts.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
