# Artifact Manifest

Packet: `31193-c1-task41-hnsw-build-heap-relation-guards`

Head SHA: `a1478b4f2fc2f6f733332f715276b9f0ab102d1b`

Timestamp: `2026-05-17T06:44:29Z`

## Artifacts

### validation.md

- Lane: Task 41 shared relation guard consolidation
- Fixture: static Rust safety/refactor validation
- Storage format: not applicable
- Rerank mode: not applicable
- Isolated one-index-per-table vs shared-table surface: not applicable
- Command summary:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `make fmt-check`
  - `bash scripts/unsafe_baseline_report.sh`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4241 entries`
  - `entries: 4241`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 5.29s`
