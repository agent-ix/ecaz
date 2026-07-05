# Artifact manifest

- Head SHA: `7e589803f97636a53e0e2dfe0ee594757a9d3a73`
- Task bucket: `reviews/task-39/030-local-store-edge-cases`
- Lane: `SpireLocalObjectStore` constructor + top-graph epoch guard
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `local-store-focused-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml
  --lib local_object_store_rejects_invalid_store_and_epoch`
- Timestamp: 2026-05-19
- Key result: 1 passed; 0 failed.
