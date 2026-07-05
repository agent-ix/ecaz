# Artifact Manifest

- head SHA: `69807c26ebc1bf5b69cdd02575f9db024a9bc2eb`
- packet/topic: `31197-c1-task41-hnsw-debug-snapshot-slot-guards`
- lane / fixture / storage format / rerank mode: Task 41 unsafe resource wrapper consolidation; HNSW debug heap-backed scan helpers; N/A; N/A
- command used:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `make fmt-check`
  - `bash scripts/unsafe_baseline_report.sh`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: `2026-05-17T15:36:17Z`
- isolated one-index-per-table or shared-table surfaces: N/A; static Rust/unsafe-baseline validation only
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4237 entries`
  - `entries: 4237`
  - `cargo check --all-targets --no-default-features --features pg18,bench` completed successfully
  - Known non-fatal output: stable rustfmt warnings for unstable formatting options, PG18 C header unused-parameter warnings, and existing `src/am/mod.rs` unused re-export warning.

See `validation.md` for the copied validation summary.
