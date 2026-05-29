# Artifact Manifest: Task 64 HNSW Codec Build Sizing

- head SHA: `1eac01c43f81238656ea063f15e2ec019ad7a6ed`
- task bucket: `reviews/task-64/`
- packet path: `reviews/task-64/002-hnsw-codec-build-sizing/`
- timestamp: `2026-05-26T23:05:26Z`
- lane: HNSW codec-adapter build sizing
- fixture: Rust compile validation only
- storage formats: `turboquant`, `pq_fastscan`
- rerank mode: unchanged
- isolated one-index-per-table or shared-table surface: not applicable

## Commands

### `cargo check -q --lib`

- command: `cargo check -q --lib`
- result: passed with exit code 0
- key result lines: no stdout/stderr output
