# Artifact Manifest

- Packet: `9047-task42-ivf-fixtures`
- Head SHA: `0852a7d440defe89c1113138aa7de60e3fd3cb58`
- Timestamp: `2026-05-17T20:38:06Z`
- Lane: Task 42 IVF on-disk fixture decode slice
- Fixture: `fixtures/on-disk/ivf_metadata_v1.hex`, `fixtures/on-disk/ivf_centroid_tuple_v1.hex`, `fixtures/on-disk/ivf_list_directory_tuple_v1.hex`, `fixtures/on-disk/ivf_posting_tuple_v1.hex`, `fixtures/on-disk/ivf_pq_codebook_tuple_v1.hex`
- Storage format: IVF v1 metadata, centroid, list-directory, posting, and PQ-codebook tuple payloads
- Rerank mode: IVF metadata fixture uses persisted `heap_f32`; posting fixture carries a rerank TID
- Shared-table vs isolated-table: not applicable; pure codec fixture tests

## Artifacts

- `make-on-disk-fixtures.log`
  - Command: `make on-disk-fixtures`
  - Key result: `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Byte-swapped rejection covered: HNSW metadata v3 format-version bytes,
    DiskANN Vamana metadata v3 format-version bytes, DiskANN Vamana node
    `neighbor_count` bytes, IVF metadata v1 format-version bytes, and IVF
    centroid tuple `dimensions` bytes.
- `make-layout-check.log`
  - Command: `make layout-check`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

Both commands emitted the pre-existing unused-import warning in `src/am/mod.rs`.
