# Artifact Manifest

- Packet: `9046-task42-tuple-fixtures`
- Head SHA: `88069ee2c2a657a5350ae7f95c3e1e225579021b`
- Timestamp: `2026-05-17T20:31:15Z`
- Lane: Task 42 on-disk tuple/codebook fixture decode slice
- Fixture: `fixtures/on-disk/hnsw_element_tuple_v3.hex`, `fixtures/on-disk/hnsw_neighbor_tuple_v3.hex`, `fixtures/on-disk/hnsw_grouped_codebook_tuple_v3.hex`, `fixtures/on-disk/diskann_vamana_node_tuple_v3.hex`, `fixtures/on-disk/diskann_vamana_codebook_tuple_v3.hex`
- Storage format: HNSW v3 element/neighbor/grouped-codebook tuple payloads; DiskANN Vamana v3 node/codebook tuple payloads
- Rerank mode: not applicable
- Shared-table vs isolated-table: not applicable; pure codec fixture tests

## Artifacts

- `make-on-disk-fixtures.log`
  - Command: `make on-disk-fixtures`
  - Key result: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Byte-swapped rejection covered: HNSW metadata v3 format-version bytes,
    DiskANN Vamana metadata v3 format-version bytes, DiskANN Vamana node
    `neighbor_count` bytes.
- `make-layout-check.log`
  - Command: `make layout-check`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

Both commands emitted the pre-existing unused-import warning in `src/am/mod.rs`.
