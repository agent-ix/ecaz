# Artifact Manifest

## Packet

- packet: `31181-c1-task41-hnsw-scan-debug-index-guards`
- head SHA: `62d8a4324dc5550e3e0eaa79bd0c924668dd1a06`
- timestamp: `2026-05-17T04:24:56Z`
- lane / fixture / storage format / rerank mode: N/A, static safety-hardening slice
- surface isolation: N/A, no PostgreSQL benchmark or index surface executed

## Artifacts

### validation.md

- command set:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `make fmt-check`
  - `bash scripts/unsafe_baseline_report.sh`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4284 entries`
  - `entries: 4284`
  - `461 src/am/ec_hnsw/scan_debug.rs`
  - `Finished dev profile [unoptimized + debuginfo] target(s)`
- notes:
  - `cargo fmt` / `make fmt-check` emitted the stable rustfmt warnings for unstable import grouping options.
  - `cargo check` emitted existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.
