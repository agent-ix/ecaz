# Artifact Manifest: HNSW Scan Setup and Cache Safety

Head SHA: `ce7858b391c1ba4ff246d91f9c569e49c603171e`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/012-hnsw-scan-setup-cache-safety`

Timestamp: `2026-05-19T04:03:59Z`

Lane / fixture / storage format / rerank mode: HNSW scan setup and scan-owned
cache state; no benchmark fixture or rerank lane.

Artifacts:

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,329 entries across 102 files;
    `src/am/ec_hnsw/scan.rs` had 258 entries.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: expected failure from shifted `src/am/ec_hnsw/scan.rs`
    baseline line numbers before the refresh.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - key result: patch context for the HNSW scan setup/cache changes before
    baseline refresh.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,264
    entries.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,264 entries across 102 files;
    `src/am/ec_hnsw/scan.rs` had 193 entries.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - key result: passed; emitted existing stable-toolchain warnings about
    unstable rustfmt options.
- `unsafe-baseline-update-after-fmt.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,264
    entries after formatting.
- `unsafe-baseline-final.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,264 entries across 102 files.
- `unsafe-audit-final.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: failed with borrow error `E0499` from the first graph-element
    cache helper version.
- `cargo-fmt-after-borrow-fix.log`
  - command: `cargo fmt --all`
  - key result: passed; emitted existing stable-toolchain warnings about
    unstable rustfmt options.
- `unsafe-baseline-update-after-borrow-fix.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - key result: wrote `scripts/unsafe_comment_baseline.txt` with 3,264
    entries after the borrow fix.
- `unsafe-baseline-final-after-borrow-fix.log`
  - command: `make unsafe-baseline-report`
  - key result: 3,264 entries across 102 files;
    `src/am/ec_hnsw/scan.rs` had 193 entries.
- `unsafe-audit-final-after-borrow-fix.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: passed with no output.
- `cargo-check-pg18-bench-after-borrow-fix.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - key result: passed with existing warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
