# Artifact Manifest

- head SHA: `0a332ef5b6f15a2acc74e29b55f45bc565aaafa3`
- packet/topic: `31198-c1-task41-shared-index-scan-guard`
- lane / fixture / storage format / rerank mode: Task 41 unsafe resource wrapper consolidation; SPIRE planner SQL-placement scan and HNSW debug heap-backed scan helpers; N/A; N/A
- command used:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `make fmt-check`
  - `bash scripts/unsafe_baseline_report.sh`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: `2026-05-17T15:43:03Z`
- isolated one-index-per-table or shared-table surfaces: N/A; static Rust/unsafe-baseline validation only
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4235 entries`
  - `entries: 4235`
  - `cargo check --all-targets --no-default-features --features pg18,bench` completed successfully
  - Known non-fatal output: stable rustfmt warnings for unstable formatting options, PG18 C header unused-parameter warnings, and existing `src/am/mod.rs` unused re-export warning.

See `validation.md` for the copied validation summary.
