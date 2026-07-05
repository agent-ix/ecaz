# Task 50 Packet 135 Artifact Manifest

- Head SHA: `68a7d617a4dbbfd38526f74c5ff8d61e3ce1574e`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/135-type-info-boundary`
- Timestamp: `2026-05-20T18:53:19-07:00`
- Lane: unsafe burndown / PostgreSQL type metadata boundary
- Fixture: local source tree
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: source-level unsafe ledger, not a benchmark surface

## Artifacts

### `artifacts/code-stat.log`

- Command: `git show --stat --oneline HEAD > reviews/task-50/135-type-info-boundary/artifacts/code-stat.log`
- Purpose: committed file summary for code commit `68a7d617a4dbbfd38526f74c5ff8d61e3ce1574e`.

### `artifacts/code-diff.patch`

- Command: `git show --patch --stat HEAD > reviews/task-50/135-type-info-boundary/artifacts/code-diff.patch`
- Purpose: packet-local patch evidence for the code commit under review.

### `artifacts/git-diff-check.log`

- Command: `git diff --check > reviews/task-50/135-type-info-boundary/artifacts/git-diff-check.log`
- Result: passed with no output.

### `artifacts/cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench > reviews/task-50/135-type-info-boundary/artifacts/cargo-check-pg18-bench.log 2>&1`
- Result: passed.
- Note: log contains the known pre-existing `src/am/mod.rs` unused import warning.

### `artifacts/src-unsafe-block-count-after.log`

- Command: `make unsafe-block-count > reviews/task-50/135-type-info-boundary/artifacts/src-unsafe-block-count-after.log`
- Key result: `unsafe_blocks 1563`, `files 121`.

### `artifacts/count-summary.md`

- Command: `awk '{s+=$1; f+=1} END {print "unsafe_blocks " s; print "files " f}' reviews/task-50/135-type-info-boundary/artifacts/src-unsafe-block-count-after.log > reviews/task-50/135-type-info-boundary/artifacts/count-summary.md`
- Key result: `unsafe_blocks 1563`, `files 121`.

### `artifacts/unsafe-ledger-after.jsonl`

- Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/135-type-info-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/135-type-info-boundary > reviews/task-50/135-type-info-boundary/artifacts/unsafe-ledger-generate.log 2>&1`
- Key result: generated `1563` unsafe ledger rows.

### `artifacts/unsafe-ledger-generate.log`

- Command log for unsafe ledger generation.
- Key result: `wrote 1563 unsafe ledger rows to reviews/task-50/135-type-info-boundary/artifacts/unsafe-ledger-after.jsonl`.

### `artifacts/unsafe-ledger-check.log`

- Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/135-type-info-boundary/artifacts/unsafe-ledger-after.jsonl > reviews/task-50/135-type-info-boundary/artifacts/unsafe-ledger-check.log 2>&1`
- Key result: `ledger covers 1563 current unsafe rows`.

## Notes

- This was not a benchmark or measurement packet, so isolated/shared table
  surfaces do not apply.
- The pre-existing dirty files outside this slice were not staged:
  `src/am/ec_ivf/build.rs`, `src/am/ec_ivf/page.rs`, `src/am/ec_ivf/scan.rs`,
  `src/quant/simd.rs`, and `callgrind.out.1807567`.
