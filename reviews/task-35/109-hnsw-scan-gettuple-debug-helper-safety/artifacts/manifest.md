# Task 35 Packet 109 Artifact Manifest

- Task bucket: `reviews/task-35/`
- Packet: `reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/`
- Head SHA summarized: `b86d7b2bde79867915d51ed2202fb8fafd09b600`
- Lane: unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable
- Surface isolation: not applicable; test-helper refactor and static validation only

## Summary

- Global unsafe-comment baseline moved from `416` entries across `34` files to `362` entries across `33` files.
- `src/tests/ec_hnsw_scan_gettuple.rs` moved from `54` baseline entries to `0`.
- Remaining baseline entries are test-only under `src/tests/`.

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/unsafe-baseline-report-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 416`, `files: 34`, `54 src/tests/ec_hnsw_scan_gettuple.rs`.

### `hnsw-scan-gettuple-baseline-before.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/ec_hnsw_scan_gettuple.rs\")==1{print ++n \":\" $0}' scripts/unsafe_comment_baseline.txt" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/hnsw-scan-gettuple-baseline-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `54` entries in `src/tests/ec_hnsw_scan_gettuple.rs`.

### `unsafe-audit-before.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/unsafe-audit-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-update-after-format.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh --update-baseline" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/unsafe-baseline-update-after-format.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `wrote scripts/unsafe_comment_baseline.txt with 362 entries`.

### `unsafe-audit-after.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/unsafe-audit-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-report-after.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/unsafe-baseline-report-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 362`, `files: 33`.

### `hnsw-scan-gettuple-baseline-after.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/ec_hnsw_scan_gettuple.rs\")==1{print ++n \":\" $0} END{if(n==0) print \"entries: 0\"}' scripts/unsafe_comment_baseline.txt" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/hnsw-scan-gettuple-baseline-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `entries: 0`.

### `git-diff-check.log`

- Command: `script -q -c "git diff --check" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/git-diff-check.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `diff-after-format.patch`

- Command: `git diff -- src/tests/ec_hnsw_scan_gettuple.rs scripts/unsafe_comment_baseline.txt > reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/diff-after-format.patch`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: patch snapshot after `cargo fmt --all` and baseline regeneration.

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-35/109-hnsw-scan-gettuple-debug-helper-safety/artifacts/cargo-check-pg18-bench.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.
- Known warnings: unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`; unused SPIRE imports/re-exports in `src/am/mod.rs`.
