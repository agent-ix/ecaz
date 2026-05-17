# Artifact Manifest

- Packet: `9042-task42-on-disk-layout-contracts`
- Head SHA: `6badde59124944181a6b8e10624ff8afffc1d061`
- Timestamp: `2026-05-17T13:02:12-07:00`
- Lane: Task 42 static on-disk layout assertion slice
- Fixture: not applicable; no on-disk golden fixtures added in this slice
- Storage format: generic page storage, HNSW metadata/tuples, DiskANN metadata
- Rerank mode: not applicable
- Shared-table vs isolated-table: not applicable; compile-only layout check

## Artifacts

- `cargo-test-size-of-assertions.log`
  - Command: `cargo test --features bench --test size_of_assertions`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Note: run emitted one pre-existing library unused-import warning in
    `src/am/mod.rs`.
