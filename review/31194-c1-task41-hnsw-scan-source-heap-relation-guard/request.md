# Task 41 Review Request: HNSW Scan Source Heap Relation Guard

## Scope

This checkpoint migrates the HNSW scan source-kind helper
`index_has_default_heap_f32_source` from manual `table_open` /
`table_close` to `HeapRelationGuard`.

Touched file:

- `src/am/ec_hnsw/scan.rs`

Code commit: `3c21ef2149ac588d34bb127c7b32d3f0a38357f0`

## Safety Invariant

The helper opens the heap relation with AccessShare only when the index
relation resolves to a valid heap relation. `HeapRelationGuard` owns the
matching close, and its lifetime covers the call to
`source::resolve_indexed_vector_attribute`.

The longer-lived grouped heap-rerank scan-state relation remains for a
separate resource-state slice because it stores relation, snapshot, and slot
state across scan operations.

## Baseline Impact

Unsafe comment baseline decreased:

- before: `4241`
- after: `4239`

This removes two tracked unsafe sites from the simple HNSW scan source check.

## Validation

See `artifacts/validation.md`.

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Review Focus

- Confirm AccessShare behavior is preserved.
- Confirm deferring the grouped heap-rerank relation/snapshot/slot bundle to
  a separate resource-state slice is the right boundary.
