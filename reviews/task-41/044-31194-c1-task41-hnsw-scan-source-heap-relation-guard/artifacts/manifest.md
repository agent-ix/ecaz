# Artifact Manifest

Packet: `31194-c1-task41-hnsw-scan-source-heap-relation-guard`

Head SHA: `3c21ef2149ac588d34bb127c7b32d3f0a38357f0`

Timestamp: `2026-05-17T08:42:50Z`

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
  - `wrote scripts/unsafe_comment_baseline.txt with 4239 entries`
  - `entries: 4239`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 5.08s`
