# Manifest: Task 41 Invariant #2 HNSW source slot Datum lifetime audit

- head SHA: `2539dba25b72e7e2497a579c912c82e0fa560c30`
- task bucket and packet path:
  `reviews/task-41/126-hnsw-source-slot-datum-lifetime-audit/`
- lane / fixture / storage format / rerank mode: source audit; no SQL fixture,
  storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:11:10Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; no
  benchmark or SQL execution.

## Artifacts

### hnsw-slot-callers.log

- command used:
  `rg -n "required_slot_datum\\(|with_indexed_ecvector_from_slot\\(|with_source_from_heap_row\\(|with_flat_float4_source_from_datum\\(|ExecClearTuple\\(|heap_getnextslot\\(" src/am/ec_hnsw -g '*.rs'`
- key result lines:
  - `src/am/ec_hnsw/source.rs:677`: `with_source_from_heap_row` reads the
    source Datum.
  - `src/am/ec_hnsw/source.rs:678`: source Datum is consumed by
    `with_flat_float4_source_from_datum`.
  - `src/am/ec_hnsw/build.rs:754` and `757`: build scan reads indexed and
    source Datums from the live heap slot.
  - `src/am/ec_hnsw/build.rs:764`: source Datum is copied through the closure
    before building the owned tuple.
  - `src/am/ec_hnsw/scan.rs:2596-2625`: grouped heap rerank computes the score
    through the closure before `ExecClearTuple`.

### source-slot-helper-excerpt.log

- command used:
  `sed -n '459,492p' src/am/ec_hnsw/source.rs`
- key result lines:
  - `fetch_heap_row_version` clears the slot before fetching a new heap tuple.
  - `required_slot_datum` materializes attributes, rejects NULL, and returns
    the raw Datum.

### source-closure-helper-excerpt.log

- command used:
  `sed -n '664,692p' src/am/ec_hnsw/source.rs`
- key result lines:
  - `with_flat_float4_source_from_datum` uses a higher-ranked closure.
  - `with_source_from_heap_row` forwards the slot Datum directly into that
    closure.
  - `with_indexed_ecvector_from_slot` also closure-scopes the slot-backed
    ecvector view.

### build-source-slot-excerpt.log

- command used:
  `sed -n '744,779p' src/am/ec_hnsw/build.rs`
- key result lines:
  - build scan reads the indexed and source Datums from the active slot.
  - the source vector is copied to `Vec<f32>` inside the closure.
  - `build_heap_tuple_with_source` consumes the indexed Datum before the next
    heap-scan slot reuse.

### scan-rerank-slot-excerpt.log

- command used:
  `sed -n '2592,2627p' src/am/ec_hnsw/scan.rs`
- key result lines:
  - grouped heap rerank reads the slot Datum inside
    `with_flat_float4_source_from_datum`.
  - only the scalar score escapes the closure.
  - the slot is cleared after the closure returns.

### git-status.log

- command used:
  `git status --short --branch`
- key result lines:
  - branch was `task41-invariant2-lifetimes`.
  - only the new HNSW audit packet was untracked when the audit artifacts were
    captured.
