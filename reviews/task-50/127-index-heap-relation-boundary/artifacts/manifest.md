---
head_sha: 98142cb52ae9e3ad6b868e51b3fdab4227059b19
task_bucket: reviews/task-50
packet: reviews/task-50/127-index-heap-relation-boundary
timestamp: 2026-05-20T18:17:08-07:00
lane: unsafe-burndown
fixture: static-analysis
storage_format: n/a
rerank_mode: n/a
isolation: n/a - static compile and ledger validation only
---

# Artifact Manifest

## code-diff.patch

- Command: `git show --format=fuller --stat --patch 98142cb5 > reviews/task-50/127-index-heap-relation-boundary/artifacts/code-diff.patch`
- Result: records code commit `98142cb52ae9e3ad6b868e51b3fdab4227059b19`.
- Key lines: extends `src/storage/relation.rs` with relation OID and index-heap OID helpers, then migrates SPIRE, HNSW, and DiskANN call sites.

## git-diff-check.log

- Command: `git diff --check 98142cb5^ 98142cb5 > reviews/task-50/127-index-heap-relation-boundary/artifacts/git-diff-check.log`
- Result: pass; log is empty.

## src-unsafe-block-count-after.log

- Command: `make unsafe-block-count > reviews/task-50/127-index-heap-relation-boundary/artifacts/src-unsafe-block-count-after.log`
- Result: after this slice, `src/` has `1596` unsafe blocks across `123` files.
- Key count change: previous packet 126 recorded `1607` unsafe blocks across `123` files.

## count-summary.md

- Command: `make unsafe-block-count | awk '{s+=$1; f+=1} END {print "unsafe_blocks " s; print "files " f}' > reviews/task-50/127-index-heap-relation-boundary/artifacts/count-summary.md`
- Result:
  - `unsafe_blocks 1596`
  - `files 123`

## cargo-check-pg18-bench.log

- Command: `cargo check --all-targets --no-default-features --features pg18,bench > reviews/task-50/127-index-heap-relation-boundary/artifacts/cargo-check-pg18-bench.log 2>&1`
- Result: pass.
- Note: cargo still reports the known pre-existing `src/am/mod.rs` SPIRE DML unused-import warning.

## unsafe-ledger-after.jsonl

- Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/127-index-heap-relation-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/127-index-heap-relation-boundary > reviews/task-50/127-index-heap-relation-boundary/artifacts/unsafe-ledger-generate.log 2>&1`
- Result: generated packet-local unsafe ledger for current `src/` state.

## unsafe-ledger-check.log

- Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/127-index-heap-relation-boundary/artifacts/unsafe-ledger-after.jsonl > reviews/task-50/127-index-heap-relation-boundary/artifacts/unsafe-ledger-check.log 2>&1`
- Result: `ledger covers 1596 current unsafe rows`.
