# Artifact Manifest

- head SHA: `a374f201ac347934244e35912cc8d66d5d9aa399`
- packet/topic: `31201-c1-task41-hnsw-frontier-debug-relation-guards`
- lane / fixture / storage format / rerank mode: Task 41 unsafe resource wrapper consolidation; HNSW pg-test frontier/lifecycle debug helper relation opens; N/A; N/A
- command used:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `make fmt-check`
  - `bash scripts/unsafe_baseline_report.sh`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: `2026-05-17T16:44:56Z`
- isolated one-index-per-table or shared-table surfaces: N/A; static Rust/unsafe-baseline validation only
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4188 entries`
  - `entries: 4188`
  - `cargo check --all-targets --no-default-features --features pg18,bench` completed successfully
  - Known non-fatal output: stable rustfmt warnings for unstable formatting options, PG18 C header unused-parameter warnings, and existing `src/am/mod.rs` unused re-export warning.

See `validation.md` for the copied validation summary.
