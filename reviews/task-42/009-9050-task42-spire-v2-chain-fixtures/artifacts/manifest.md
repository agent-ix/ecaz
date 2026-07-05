# Artifact Manifest

- head SHA: `5c48a976f54a1af1b2eba5f301a9332e5e95ea36`
- packet/topic: `9050-task42-spire-v2-chain-fixtures`
- timestamp: `2026-05-17T21:22:58Z`
- storage surface: shared-table source tree fixture decode checks; no PostgreSQL cluster or index-table isolation involved
- rerank mode: not applicable

## Artifacts

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite, including new SPIRE V2 leaf meta/segment and generic V2 chain meta/segment fixtures
- storage format: checked by fixture-specific decoders; new SPIRE fixtures use `PARTITION_OBJECT_FORMAT_VERSION_V2`
- command used: `script -q -c "make on-disk-fixtures" reviews/task-42/009-9050-task42-spire-v2-chain-fixtures/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 40 tests`
  - `test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" reviews/task-42/009-9050-task42-spire-v2-chain-fixtures/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- notes:
  - The run emitted the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.
