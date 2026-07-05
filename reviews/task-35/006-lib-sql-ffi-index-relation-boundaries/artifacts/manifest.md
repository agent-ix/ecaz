# Artifact Manifest: lib.rs SQL FFI Index Relation Boundaries

Head SHA: `bd523a12c0577f43cf2e93453196c1e37b4ff210`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/006-lib-sql-ffi-index-relation-boundaries`

Timestamp: `2026-05-19T03:16:24Z`

Lane / fixture / storage format / rerank mode: not applicable; static unsafe
burndown and Rust compile validation.

Artifacts:

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,657 entries across 107 files; `src/lib.rs` had 181 entries.
- `audit-before.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: failed before baseline refresh after source line movement and
    unsafe-site removal.
- `fmt.log`
  - command: `cargo fmt --all`
  - key result: passed; emitted existing stable-toolchain warnings about
    unstable rustfmt options.
- `cargo-check-pg18.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: passed with existing warnings in `src/am/common/parallel.rs`
    and `src/am/mod.rs`.
- `update-baseline.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,579
    entries.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,579 entries across 107 files; `src/lib.rs` has 103 entries.
- `audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
