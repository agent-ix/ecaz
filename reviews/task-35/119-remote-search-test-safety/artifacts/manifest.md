# Task 35 Packet 119 Artifact Manifest

- Task bucket: `reviews/task-35/`
- Packet: `reviews/task-35/119-remote-search-test-safety/`
- Head SHA summarized: `72b366b144aa2f584a119585ef66c03b5fbe292a`
- Lane: unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable
- Surface isolation: not applicable; test-only remote-search debug state mutation and static validation only

## Summary

- Global unsafe-comment baseline moved from `119` entries across `23` files to `60` entries across `13` files.
- `src/tests/remote_search/*` moved from `59` baseline entries to `0`.
- Remaining baseline entries are test-only under `src/tests/`.

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/119-remote-search-test-safety/artifacts/unsafe-baseline-report-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 119`, `files: 23`, remote-search files total `59`.

### `remote-search-baseline-before.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/remote_search/\")==1{print ++n \":\" $0}' scripts/unsafe_comment_baseline.txt" reviews/task-35/119-remote-search-test-safety/artifacts/remote-search-baseline-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `59` entries under `src/tests/remote_search/`.

### `unsafe-audit-before.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/119-remote-search-test-safety/artifacts/unsafe-audit-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-update-after-format.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh --update-baseline" reviews/task-35/119-remote-search-test-safety/artifacts/unsafe-baseline-update-after-format.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `wrote scripts/unsafe_comment_baseline.txt with 60 entries`.

### `unsafe-audit-after.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/119-remote-search-test-safety/artifacts/unsafe-audit-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-report-after.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/119-remote-search-test-safety/artifacts/unsafe-baseline-report-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 60`, `files: 13`.

### `remote-search-baseline-after.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/remote_search/\")==1{print ++n \":\" $0} END{if(n==0) print \"entries: 0\"}' scripts/unsafe_comment_baseline.txt" reviews/task-35/119-remote-search-test-safety/artifacts/remote-search-baseline-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `entries: 0`.

### `git-diff-check.log`

- Command: `script -q -c "git diff --check" reviews/task-35/119-remote-search-test-safety/artifacts/git-diff-check.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `diff-after-format.patch`

- Command: `git diff -- src/tests/remote_search scripts/unsafe_comment_baseline.txt > reviews/task-35/119-remote-search-test-safety/artifacts/diff-after-format.patch`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: patch snapshot after `cargo fmt --all` and baseline regeneration.

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-35/119-remote-search-test-safety/artifacts/cargo-check-pg18-bench.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.
- Known warnings: unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`; unused SPIRE imports/re-exports in `src/am/mod.rs`.
