# Artifact Manifest

## Packet

- packet: `31182-c1-task41-hnsw-debug-heap-scan-resource-guards`
- head SHA: `a9589ed432e7e1488d3b42996680173a5c3b9e13`
- timestamp: `2026-05-17T04:30:29Z`
- lane / fixture / storage format / rerank mode: N/A, static safety-hardening slice
- surface isolation: N/A, no PostgreSQL benchmark or index surface executed

## Artifacts

### validation.md

- command set:
  - `cargo fmt`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `bash scripts/unsafe_baseline_report.sh`
  - `make fmt-check`
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4265 entries`
  - `entries: 4265`
  - `442 src/am/ec_hnsw/scan_debug.rs`
  - `Finished dev profile [unoptimized + debuginfo] target(s)`
- notes:
  - `cargo fmt` / `make fmt-check` emitted the stable rustfmt warnings for unstable import grouping options.
  - `cargo check` emitted existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.
