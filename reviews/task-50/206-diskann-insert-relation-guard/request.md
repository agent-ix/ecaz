# Task 50 Packet 206: DiskANN Insert Relation Guard

## Summary

This slice adds a borrowed `DiskannInsertRelation` guard for DiskANN insert-time page and metadata helpers. The unsafe relation-liveness assertion is now made once at callback/diagnostic boundaries, while the insert helpers use safe methods on the guard for buffer reads, main-fork block counts, and generic WAL startup.

The slice converts these DiskANN insert helpers away from raw `pg_sys::Relation` unsafe function signatures:

- `read_metadata_page`
- `with_locked_metadata_page`
- `bootstrap_empty_insert_output`
- `bind_duplicate_heap_tid`
- `append_live_node`
- `add_backlinks_if_free`
- `apply_backlink_mutations`
- `increment_inserted_since_rebuild`

The remaining unsafe blocks in `insert.rs` are page-byte and PostgreSQL FFI boundaries where the helper still constructs or mutates raw page memory.

## Counts

- `src/am/ec_diskann/insert.rs`: `37 -> 17` unsafe references.
- `src`: `2594 -> 2575` unsafe references since packet 205.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_diskann/insert.rs src/am/ec_diskann/routine.rs src/am/ec_diskann/cost.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
