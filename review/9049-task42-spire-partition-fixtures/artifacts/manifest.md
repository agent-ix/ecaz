# Artifact Manifest

- head SHA: `a96b65669cb289439cb4892eb321e5e99ffd0238`
- packet/topic: `9049-task42-spire-partition-fixtures`
- timestamp: `2026-05-17T20:54:28Z`
- storage surface: shared-table source tree fixture decode checks; no PostgreSQL cluster or index-table isolation involved
- rerank mode: not applicable

## Artifacts

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite, including new SPIRE V1 leaf/routing/delta/top-graph partition object fixtures
- storage format: checked by fixture-specific decoders; SPIRE partition objects use `PARTITION_OBJECT_FORMAT_VERSION_V1`
- command used: `script -q -c "make on-disk-fixtures" review/9049-task42-spire-partition-fixtures/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 34 tests`
  - `test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" review/9049-task42-spire-partition-fixtures/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.
