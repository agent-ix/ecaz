# Task 35 Packet 112 Artifact Manifest

- Task bucket: `reviews/task-35/`
- Packet: `reviews/task-35/112-dml-frontdoor-test-helper-safety/`
- Head SHA summarized: `04b1e061eb6d095b88de57b2e095aba1198fbe90`
- Lane: unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable
- Surface isolation: not applicable; test-helper refactor and static validation only

## Summary

- Global unsafe-comment baseline moved from `281` entries across `31` files to `252` entries across `30` files.
- `src/tests/dml_frontdoor.rs` moved from `29` baseline entries to `0`.
- Remaining baseline entries are test-only under `src/tests/`.

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/unsafe-baseline-report-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 281`, `files: 31`, `29 src/tests/dml_frontdoor.rs`.

### `dml-frontdoor-baseline-before.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/dml_frontdoor.rs\")==1{print ++n \":\" $0}' scripts/unsafe_comment_baseline.txt" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/dml-frontdoor-baseline-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `29` entries in `src/tests/dml_frontdoor.rs`.

### `unsafe-audit-before.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/unsafe-audit-before.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-update-after-format.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh --update-baseline" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/unsafe-baseline-update-after-format.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `wrote scripts/unsafe_comment_baseline.txt with 252 entries`.

### `unsafe-audit-after.log`

- Command: `script -q -c "bash scripts/check_unsafe_comments.sh" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/unsafe-audit-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `unsafe-baseline-report-after.log`

- Command: `script -q -c "bash scripts/unsafe_baseline_report.sh" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/unsafe-baseline-report-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key lines: `entries: 252`, `files: 30`.

### `dml-frontdoor-baseline-after.log`

- Command: `script -q -c "awk 'index($0,\"src/tests/dml_frontdoor.rs\")==1{print ++n \":\" $0} END{if(n==0) print \"entries: 0\"}' scripts/unsafe_comment_baseline.txt" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/dml-frontdoor-baseline-after.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Key line: `entries: 0`.

### `git-diff-check.log`

- Command: `script -q -c "git diff --check" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/git-diff-check.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.

### `diff-after-format.patch`

- Command: `git diff -- src/tests/dml_frontdoor.rs scripts/unsafe_comment_baseline.txt > reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/diff-after-format.patch`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: patch snapshot after `cargo fmt --all` and baseline regeneration.

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-35/112-dml-frontdoor-test-helper-safety/artifacts/cargo-check-pg18-bench.log`
- Timestamp: `2026-05-19 America/Los_Angeles`
- Result: exited `0`.
- Known warnings: unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`; unused SPIRE imports/re-exports in `src/am/mod.rs`.
