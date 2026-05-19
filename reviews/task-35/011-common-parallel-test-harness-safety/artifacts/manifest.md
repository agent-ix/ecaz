# Artifact Manifest: Common Parallel Test Harness Safety

Head SHA: `8e5f7eba878ca152069d722ad94fb8a2b7602e0a`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/011-common-parallel-test-harness-safety`

Timestamp: `2026-05-19T03:51:04Z`

Lane / fixture / storage format / rerank mode: not applicable; static unsafe
test-harness refactor and Rust compile validation.

Artifacts:

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,404 entries across 103 files;
    `src/am/common/parallel.rs` had 75 entries.
- `unsafe-audit-before.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,329
    entries.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,329 entries across 102 files;
    `src/am/common/parallel.rs` had 0 entries.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - key result: passed; emitted existing stable-toolchain warnings about
    unstable rustfmt options.
- `unsafe-baseline-update-after-fmt.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,329
    entries after formatting.
- `unsafe-baseline-final.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,329 entries across 102 files;
    `src/am/common/parallel.rs` had 0 entries.
- `unsafe-audit-final.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-test-parallel-scan.log`
  - command: `cargo test --lib parallel_scan --no-default-features --features pg18,bench`
  - key result: compiled, then failed before running tests with
    `undefined symbol: LockBuffer`.
- `cargo-test-parallel-scan-no-run.log`
  - command: `cargo test --lib parallel_scan --no-run --no-default-features --features pg18,bench`
  - key result: passed.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: passed with existing warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
