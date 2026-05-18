# Artifact Manifest

- head SHA: `5ecb486ab7a89af5e7b892d740a652aa524be3d5`
- packet/topic: `9054-task42-ci-fixture-lanes`
- timestamp: `2026-05-17T21:37:01Z`
- storage surface: CI wiring plus source tree fixture/matrix checks
- rerank mode: not applicable

## Artifacts

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite
- storage format: current on-disk fixture formats
- command used: `script -q -c "make on-disk-fixtures" reviews/task-42/013-9054-task42-ci-fixture-lanes/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 45 tests`
  - `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

### `make-upgrade-smoke.log`

- lane: on-disk format-version compatibility matrix smoke
- fixture: `fixtures/upgrade/matrix.csv` and referenced on-disk fixtures
- storage format: current HNSW, DiskANN, IVF, and SPIRE partition object format tags
- command used: `script -q -c "make upgrade-smoke" reviews/task-42/013-9054-task42-ci-fixture-lanes/artifacts/make-upgrade-smoke.log`
- key result lines:
  - `running 2 tests`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" reviews/task-42/013-9054-task42-ci-fixture-lanes/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

All three logs emit the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.
