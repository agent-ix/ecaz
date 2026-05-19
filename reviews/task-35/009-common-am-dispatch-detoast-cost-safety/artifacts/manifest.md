# Artifact Manifest: Common AM Dispatch Detoast and Cost Safety

Head SHA: `0e4eae93b20f7e7702d854d9b209b44360057df3`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/009-common-am-dispatch-detoast-cost-safety`

Timestamp: `2026-05-19T03:31:32Z`

Lane / fixture / storage format / rerank mode: not applicable; static unsafe
documentation and Rust compile validation.

Artifacts:

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,476 entries across 106 files.
- `audit-before.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
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
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,448
    entries.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,448 entries across 103 files; the three scoped files have 0
    entries.
- `audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
