# Artifact Manifest

- Packet: `9044-task42-spire-layout-contracts`
- Head SHA: `a2f12000c505c11b46daac9627d657b1ca071324`
- Timestamp: `2026-05-17T13:19:41-07:00`
- Lane: Task 42 static on-disk layout assertion slice
- Fixture: not applicable; no on-disk golden fixtures added in this slice
- Storage format: SPIRE partition-object storage and metadata codecs
- Rerank mode: not applicable
- Shared-table vs isolated-table: not applicable; compile-only layout check

## Artifacts

- `cargo-test-size-of-assertions.log`
  - Command: `cargo test --features bench --test size_of_assertions`
  - Key result: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - Note: run emitted one pre-existing library unused-import warning in
    `src/am/mod.rs`.
