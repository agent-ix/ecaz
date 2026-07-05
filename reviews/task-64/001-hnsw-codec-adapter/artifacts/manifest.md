# Artifact Manifest: Task 64 HNSW Codec Adapter

- head SHA: `556a11637a1673c224b36e19ca5bdd913d0651ec`
- task bucket: `reviews/task-64/`
- packet path: `reviews/task-64/001-hnsw-codec-adapter/`
- timestamp: `2026-05-26T23:03:07Z`
- lane: HNSW codec-adapter foundation
- fixture: Rust compile validation only
- storage formats: `turboquant`, `pq_fastscan`
- rerank mode: unchanged
- isolated one-index-per-table or shared-table surface: not applicable

## Commands

### `cargo check -q --lib`

- command: `cargo check -q --lib`
- result: passed with exit code 0
- key result lines: no stdout/stderr output

### `cargo test -q hnsw_storage_codec`

- command: `cargo test -q hnsw_storage_codec`
- result: failed at runtime after linking test binary
- key result lines:
  - `undefined symbol: CacheRegisterRelcacheCallback`

### `cargo test -q storage_codec_maps_reloptions_to_names --lib`

- command: `cargo test -q storage_codec_maps_reloptions_to_names --lib`
- result: failed at runtime after linking test binary
- key result lines:
  - `undefined symbol: LockBuffer`

### `cargo test --lib storage_codec_maps_reloptions_to_names -- --exact --nocapture`

- command: `cargo test --lib storage_codec_maps_reloptions_to_names -- --exact --nocapture`
- result: failed at runtime after linking test binary
- key result lines:
  - `Finished test profile`
  - `undefined symbol: LockBuffer`
