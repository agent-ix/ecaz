# Task 35 Packet 117 Artifact Manifest

- Task bucket: `reviews/task-35/`
- Packet: `reviews/task-35/117-hnsw-recall-debug-exports-test-safety/`
- Head SHA summarized: `cb8551fab7228d3861dd279766ada3f74bcb4ba9`
- Lane: unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable
- Surface isolation: not applicable; test/debug export refactor and static validation only

## Summary

- Global unsafe-comment baseline moved from `150` entries across `26` files to `132` entries across `25` files.
- `src/tests/ec_hnsw_recall_debug_exports.rs` moved from `18` baseline entries to `0`.
- Remaining baseline entries are test-only under `src/tests/`.

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/unsafe-baseline-report-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 150`, `files: 26`, `18 src/tests/ec_hnsw_recall_debug_exports.rs`.

### `hnsw-recall-debug-exports-baseline-before.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/ec_hnsw_recall_debug_exports.rs\")==1{print ++n \":\" $0}' scripts/unsafe_comment_baseline.txt" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/hnsw-recall-debug-exports-baseline-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `18` entries in `src/tests/ec_hnsw_recall_debug_exports.rs`.

### `unsafe-audit-before.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/unsafe-audit-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-update-after-format.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh --update-baseline" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/unsafe-baseline-update-after-format.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `wrote scripts/unsafe_comment_baseline.txt with 132 entries`.

### `unsafe-audit-after.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/unsafe-audit-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-report-after.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/unsafe-baseline-report-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 132`, `files: 25`.

### `hnsw-recall-debug-exports-baseline-after.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/ec_hnsw_recall_debug_exports.rs\")==1{print ++n \":\" $0} END{if(n==0) print \"entries: 0\"}' scripts/unsafe_comment_baseline.txt" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/hnsw-recall-debug-exports-baseline-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `entries: 0`.

### `git-diff-check.log`

- Command: `script -q -c "git diff --check" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/git-diff-check.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `diff-after-format.patch`

- Command: `git diff -- src/tests/ec_hnsw_recall_debug_exports.rs scripts/unsafe_comment_baseline.txt > reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/diff-after-format.patch`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: patch snapshot after `cargo fmt --all` and baseline regeneration.

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-35/117-hnsw-recall-debug-exports-test-safety/artifacts/cargo-check-pg18-bench.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.
- Known warnings: unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`; unused SPIRE imports/re-exports in `src/am/mod.rs`.
