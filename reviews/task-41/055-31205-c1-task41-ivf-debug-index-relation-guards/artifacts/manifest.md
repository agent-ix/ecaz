# Artifact Manifest

- head SHA: `aee30476e23b28063b31c96b27ece01e8147430e`
- packet/topic: `31205-c1-task41-ivf-debug-index-relation-guards`
- lane / fixture / storage format / rerank mode: Task 41 unsafe resource wrapper consolidation; IVF pg-test debug index relation helpers; N/A; N/A
- command used:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `bash scripts/unsafe_baseline_report.sh`
  - `make fmt-check`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: `2026-05-17T17:20:03Z`
- isolated one-index-per-table or shared-table surfaces: N/A; static Rust/unsafe-baseline validation only
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4146 entries`
  - `entries: 4146`
  - `cargo check --all-targets --no-default-features --features pg18,bench` completed successfully
  - Known non-fatal output: stable rustfmt warnings for unstable formatting options, PG18 C header unused-parameter warnings, and existing `src/am/mod.rs` unused re-export warning.

See `validation.md` for the copied validation summary.
