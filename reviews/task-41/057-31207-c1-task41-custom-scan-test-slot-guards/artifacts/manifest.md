# Artifact Manifest

- head SHA: `2ac2993400e6618587199363148af04b65869d61`
- packet/topic: `31207-c1-task41-custom-scan-test-slot-guards`
- lane / fixture / storage format / rerank mode: Task 41 unsafe resource wrapper consolidation; custom scan pg-test payload slot; N/A; N/A
- command used:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `bash scripts/unsafe_baseline_report.sh`
  - `make fmt-check`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: `2026-05-17T17:27:00Z`
- isolated one-index-per-table or shared-table surfaces: N/A; static Rust/unsafe-baseline validation only
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4141 entries`
  - `entries: 4141`
  - `cargo check --all-targets --no-default-features --features pg18,bench` completed successfully
  - Known non-fatal output: stable rustfmt warnings for unstable formatting options, PG18 C header unused-parameter warnings, and existing `src/am/mod.rs` unused re-export warning.

See `validation.md` for the copied validation summary.
