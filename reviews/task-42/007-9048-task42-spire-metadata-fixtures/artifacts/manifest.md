# Artifact Manifest

- Packet: `9048-task42-spire-metadata-fixtures`
- Head SHA: `82bd0565fe9a098bf6ce70f8c9ed598c22aa86b8`
- Timestamp: `2026-05-17T20:44:57Z`
- Lane: Task 42 SPIRE metadata fixture decode slice
- Fixture: `fixtures/on-disk/spire_local_store_config_v1.hex`, `fixtures/on-disk/spire_placement_entry_v1.hex`, `fixtures/on-disk/spire_placement_directory_v1.hex`, `fixtures/on-disk/spire_epoch_manifest_v1.hex`, `fixtures/on-disk/spire_manifest_entry_v1.hex`, `fixtures/on-disk/spire_object_manifest_v1.hex`
- Storage format: SPIRE metadata format v1 local-store config, placement entry/directory, epoch manifest, manifest entry, and object manifest payloads
- Rerank mode: not applicable
- Shared-table vs isolated-table: not applicable; pure codec fixture tests

## Artifacts

- `make-on-disk-fixtures.log`
  - Command: `make on-disk-fixtures`
  - Key result: `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Byte-swapped rejection covered: HNSW metadata v3 format-version bytes,
    DiskANN Vamana metadata v3 format-version bytes, DiskANN Vamana node
    `neighbor_count` bytes, IVF metadata v1 format-version bytes, IVF
    centroid tuple `dimensions` bytes, and SPIRE metadata v1 format-version
    bytes for each fixture in this packet.
- `make-layout-check.log`
  - Command: `make layout-check`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

Both commands emitted the pre-existing unused-import warning in `src/am/mod.rs`.
