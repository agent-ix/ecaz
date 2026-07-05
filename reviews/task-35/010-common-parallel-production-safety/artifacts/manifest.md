# Artifact Manifest: Common Parallel Production Safety

Head SHA: `ce380578c4cf470140d51eb7300b7377f8813a62`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/010-common-parallel-production-safety`

Timestamp: `2026-05-19T03:41:43Z`

Lane / fixture / storage format / rerank mode: not applicable; static unsafe
documentation and Rust compile validation.

Artifacts:

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,448 entries across 103 files;
    `src/am/common/parallel.rs` had 119 entries.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: expected failure from shifted
    `src/am/common/parallel.rs` test-module baseline line numbers before the
    refresh.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,404
    entries.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,404 entries across 103 files;
    `src/am/common/parallel.rs` had 75 entries.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - key result: passed; emitted existing stable-toolchain warnings about
    unstable rustfmt options.
- `unsafe-baseline-update-after-fmt.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,404
    entries after formatting.
- `unsafe-baseline-final.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,404 entries across 103 files;
    `src/am/common/parallel.rs` had 75 entries.
- `unsafe-audit-final.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: passed with existing warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
