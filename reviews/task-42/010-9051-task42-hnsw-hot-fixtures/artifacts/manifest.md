# Artifact Manifest

- head SHA: `92bd54b9f60f4b56bbe1465f1ea1a56178463056`
- packet/topic: `9051-task42-hnsw-hot-fixtures`
- timestamp: `2026-05-17T21:27:23Z`
- storage surface: shared-table source tree fixture decode checks; no PostgreSQL cluster or index-table isolation involved
- rerank mode: fixture covers HNSW cold rerank tuple bytes directly

## Artifacts

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite, including new HNSW grouped-hot, turbo-hot, and rerank tuple fixtures
- storage format: HNSW grouped-hot tuple format v2 and HNSW turbo-hot/cold tuple format v3
- command used: `script -q -c "make on-disk-fixtures" reviews/task-42/010-9051-task42-hnsw-hot-fixtures/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 43 tests`
  - `test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" reviews/task-42/010-9051-task42-hnsw-hot-fixtures/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.
