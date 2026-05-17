# Artifact Manifest

- head SHA: `289a68f62d9c1b79fd286f8e3a9a8aed6548c8cf`
- packet/topic: `9052-task42-diskann-overflow-fixture`
- timestamp: `2026-05-17T21:31:52Z`
- storage surface: shared-table source tree fixture decode checks; no PostgreSQL cluster or index-table isolation involved
- rerank mode: not applicable

## Artifacts

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite, including new DiskANN duplicate heap-TID overflow tuple fixture
- storage format: DiskANN Vamana tuple format v3
- command used: `script -q -c "make on-disk-fixtures" review/9052-task42-diskann-overflow-fixture/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 45 tests`
  - `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" review/9052-task42-diskann-overflow-fixture/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.
