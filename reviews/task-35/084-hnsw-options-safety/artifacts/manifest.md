# Task 35 Packet 084 Artifact Manifest

- Task bucket: `reviews/task-35/`
- Packet: `reviews/task-35/084-hnsw-options-safety/`
- Head SHA under review: `3004fc4a9dfe51a1670a1a6aa51d94b7874fa3b7`
- Lane: unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable
- Surface isolation: not applicable; static/code validation only

## Baseline Summary

- Before: `1768` entries across `51` files.
- File before: `src/am/ec_hnsw/options.rs` had `8` entries.
- After: `1760` entries across `50` files.
- File after: `src/am/ec_hnsw/options.rs` has `0` entries.

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `script -q -e -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/084-hnsw-options-safety/artifacts/unsafe-baseline-report-before.log`
- Timestamp: `2026-05-19 03:20:07-07:00`
- Key lines: `entries: 1768`, `files: 51`, top file includes `8 src/am/ec_hnsw/options.rs`.

### `hnsw-options-baseline-before.log`

- Command: `script -q -e -c "awk 'BEGIN{n=0} index(\$0,\"src/am/ec_hnsw/options.rs:\")==1{print \$0; n++} END{print \"entries: \" n}' scripts/unsafe_comment_baseline.txt" reviews/task-35/084-hnsw-options-safety/artifacts/hnsw-options-baseline-before.log`
- Timestamp: `2026-05-19 03:20:07-07:00`
- Key line: `entries: 8`.

### `unsafe-audit-before.log`

- Command: `script -q -e -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/084-hnsw-options-safety/artifacts/unsafe-audit-before.log`
- Timestamp: `2026-05-19 03:20:07-07:00`
- Result: passed against the existing baseline.

### `unsafe-baseline-update.log`

- Command: `script -q -e -c "bash scripts/check_unsafe_comments.sh --update-baseline" reviews/task-35/084-hnsw-options-safety/artifacts/unsafe-baseline-update.log`
- Timestamp: `2026-05-19 03:20:37-07:00`
- Key line: `wrote scripts/unsafe_comment_baseline.txt with 1760 entries`.

### `hnsw-options-baseline-after-update.log`

- Command: `script -q -e -c "awk 'BEGIN{n=0} index(\$0,\"src/am/ec_hnsw/options.rs:\")==1{print \$0; n++} END{print \"entries: \" n}' scripts/unsafe_comment_baseline.txt" reviews/task-35/084-hnsw-options-safety/artifacts/hnsw-options-baseline-after-update.log`
- Timestamp: `2026-05-19 03:20:49-07:00`
- Key line: `entries: 0`.

### `diff-before-format.patch`

- Command: `script -q -e -c "git diff -- src/am/ec_hnsw/options.rs scripts/unsafe_comment_baseline.txt" reviews/task-35/084-hnsw-options-safety/artifacts/diff-before-format.patch`
- Timestamp: `2026-05-19 03:20:58-07:00`
- Key lines: removes the 8 HNSW options baseline entries and adds reloptions safety comments.

### `cargo-fmt.log`

- Command: `script -q -e -c "cargo fmt --all" reviews/task-35/084-hnsw-options-safety/artifacts/cargo-fmt.log`
- Timestamp: `2026-05-19 03:21:02-07:00`
- Result: exited `0`; emitted existing stable-rustfmt warnings about unstable rustfmt options.

### `unsafe-baseline-update-after-fmt.log`

- Command: `script -q -e -c "bash scripts/check_unsafe_comments.sh --update-baseline" reviews/task-35/084-hnsw-options-safety/artifacts/unsafe-baseline-update-after-fmt.log`
- Timestamp: `2026-05-19 03:21:16-07:00`
- Key line: `wrote scripts/unsafe_comment_baseline.txt with 1760 entries`.

### `unsafe-audit-after.log`

- Command: `script -q -e -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/084-hnsw-options-safety/artifacts/unsafe-audit-after.log`
- Timestamp: `2026-05-19 03:21:36-07:00`
- Result: exited `0`.

### `unsafe-baseline-report-after.log`

- Command: `script -q -e -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/084-hnsw-options-safety/artifacts/unsafe-baseline-report-after.log`
- Timestamp: `2026-05-19 03:21:36-07:00`
- Key lines: `entries: 1760`, `files: 50`.

### `hnsw-options-baseline-after.log`

- Command: `script -q -e -c "awk 'BEGIN{n=0} index(\$0,\"src/am/ec_hnsw/options.rs:\")==1{print \$0; n++} END{print \"entries: \" n}' scripts/unsafe_comment_baseline.txt" reviews/task-35/084-hnsw-options-safety/artifacts/hnsw-options-baseline-after.log`
- Timestamp: `2026-05-19 03:21:36-07:00`
- Key line: `entries: 0`.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" reviews/task-35/084-hnsw-options-safety/artifacts/git-diff-check.log`
- Timestamp: `2026-05-19 03:21:36-07:00`
- Result: exited `0`.

### `cargo-check-pg18-bench.log`

- Command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-35/084-hnsw-options-safety/artifacts/cargo-check-pg18-bench.log`
- Timestamp: `2026-05-19 03:21:36-07:00`
- Result: exited `0`.
- Known unrelated warnings:
  - unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`;
  - unused SPIRE imports/re-exports in `src/am/mod.rs`.

### `final-diff.patch`

- Command: `script -q -e -c "git diff -- src/am/ec_hnsw/options.rs scripts/unsafe_comment_baseline.txt" reviews/task-35/084-hnsw-options-safety/artifacts/final-diff.patch`
- Timestamp: `2026-05-19 03:21:54-07:00`
- Key lines: final source and baseline diff for commit `3004fc4a9dfe51a1670a1a6aa51d94b7874fa3b7`.
