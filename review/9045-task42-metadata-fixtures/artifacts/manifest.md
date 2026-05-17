# Artifact Manifest

- Packet: `9045-task42-metadata-fixtures`
- Head SHA: `396bae9afacbc5afa84a89fc972667a66f2dca50`
- Timestamp: `2026-05-17T13:23:33-07:00`
- Lane: Task 42 on-disk metadata fixture decode slice
- Fixture: `fixtures/on-disk/hnsw_metadata_v3.hex`, `fixtures/on-disk/diskann_vamana_metadata_v3.hex`
- Storage format: HNSW current metadata, DiskANN Vamana metadata
- Rerank mode: not applicable
- Shared-table vs isolated-table: not applicable; pure codec fixture tests

## Artifacts

- `make-on-disk-fixtures.log`
  - Command: `make on-disk-fixtures`
  - Key result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Byte-swapped rejection covered: HNSW metadata v3 format-version bytes,
    DiskANN Vamana metadata v3 format-version bytes.
- `make-layout-check.log`
  - Command: `make layout-check`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

Both commands emitted the pre-existing unused-import warning in `src/am/mod.rs`.
