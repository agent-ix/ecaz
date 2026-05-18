# Artifact Manifest

- head SHA: `279a3311f366ebd43014381237b5269f267fa6c5`
- packet/topic: `31204-c1-task41-spire-remote-heap-resolution-guards`
- lane / fixture / storage format / rerank mode: Task 41 unsafe resource wrapper consolidation; SPIRE coordinator remote heap resolution; N/A; N/A
- command used:
  - `cargo fmt`
  - `bash scripts/check_unsafe_comments.sh --update-baseline`
  - `git diff --check`
  - `bash scripts/check_unsafe_comments.sh`
  - `bash scripts/unsafe_baseline_report.sh`
  - `make fmt-check`
  - `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: `2026-05-17T17:15:03Z`
- isolated one-index-per-table or shared-table surfaces: N/A; static Rust/unsafe-baseline validation only
- key result lines:
  - `wrote scripts/unsafe_comment_baseline.txt with 4165 entries`
  - `entries: 4165`
  - `cargo check --all-targets --no-default-features --features pg18,bench` completed successfully
  - Known non-fatal output: stable rustfmt warnings for unstable formatting options, PG18 C header unused-parameter warnings, and existing `src/am/mod.rs` unused re-export warning.

See `validation.md` for the copied validation summary.
