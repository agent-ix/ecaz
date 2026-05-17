# Artifact Manifest

- Packet: `9043-task42-ivf-diskann-layout-contracts`
- Head SHA: `046bcb246a9ccd85587fd00285f9b66018ac1b0d`
- Timestamp: `2026-05-17T13:11:25-07:00`
- Lane: Task 42 static on-disk layout assertion slice
- Fixture: not applicable; no on-disk golden fixtures added in this slice
- Storage format: DiskANN node/codebook tuples, IVF metadata and tuple codecs
- Rerank mode: not applicable
- Shared-table vs isolated-table: not applicable; compile-only layout check

## Artifacts

- `cargo-test-size-of-assertions.log`
  - Command: `cargo test --features bench --test size_of_assertions`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Note: run emitted one pre-existing library unused-import warning in
    `src/am/mod.rs`.
