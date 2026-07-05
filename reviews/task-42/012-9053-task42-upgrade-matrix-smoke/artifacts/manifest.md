# Artifact Manifest

- head SHA: `16d7f08c6bc03590db4e38cae959bc3c94d69f3a`
- packet/topic: `9053-task42-upgrade-matrix-smoke`
- timestamp: `2026-05-17T21:35:12Z`
- storage surface: source tree matrix/fixture checks; no PostgreSQL cluster or index-table isolation involved
- rerank mode: not applicable

## Artifacts

### `make-upgrade-smoke.log`

- lane: on-disk format-version compatibility matrix smoke
- fixture: `fixtures/upgrade/matrix.csv` and referenced on-disk fixtures
- storage format: current HNSW, DiskANN, IVF, and SPIRE partition object format tags
- command used: `script -q -c "make upgrade-smoke" reviews/task-42/012-9053-task42-upgrade-matrix-smoke/artifacts/make-upgrade-smoke.log`
- key result lines:
  - `running 2 tests`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - This is a source-tree matrix smoke. Historical live-cluster corpus upgrade checks are still a future extension when an incompatible format version ships.
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite
- storage format: current on-disk fixture formats
- command used: `script -q -c "make on-disk-fixtures" reviews/task-42/012-9053-task42-upgrade-matrix-smoke/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 45 tests`
  - `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" reviews/task-42/012-9053-task42-upgrade-matrix-smoke/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.
