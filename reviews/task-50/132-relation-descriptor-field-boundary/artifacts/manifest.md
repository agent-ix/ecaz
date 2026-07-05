---
head_sha: 961190cd675cdb7408b6b17af4c30daad3699cef
task_bucket: reviews/task-50
packet: reviews/task-50/132-relation-descriptor-field-boundary
timestamp: 2026-05-20T18:35:31-07:00
lane: unsafe-burndown
fixture: static-analysis
storage_format: n/a
rerank_mode: n/a
isolation: n/a - static compile and ledger validation only
---

# Artifact Manifest

## code-diff.patch

- Command: `git show --format=fuller --stat --patch 961190cd > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/code-diff.patch`
- Result: records code commit `961190cd675cdb7408b6b17af4c30daad3699cef`.
- Key lines: adds storage helpers for `reltuples` and `reltablespace`, then migrates SPIRE, IVF admin, and common planner cost callers.

## git-diff-check.log

- Command: `git diff --check 961190cd^ 961190cd > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/git-diff-check.log`
- Result: pass; log is empty.

## src-unsafe-block-count-after.log

- Command: `make unsafe-block-count > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/src-unsafe-block-count-after.log`
- Result: after this slice, `src/` has `1578` unsafe blocks across `120` files.
- Key count change: previous packet 131 recorded `1581` unsafe blocks across `121` files.

## count-summary.md

- Command: `make unsafe-block-count | awk '{s+=$1; f+=1} END {print "unsafe_blocks " s; print "files " f}' > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/count-summary.md`
- Result:
  - `unsafe_blocks 1578`
  - `files 120`

## cargo-check-pg18-bench.log

- Command: `cargo check --all-targets --no-default-features --features pg18,bench > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/cargo-check-pg18-bench.log 2>&1`
- Result: pass.
- Note: cargo still reports the known pre-existing `src/am/mod.rs` SPIRE DML unused-import warning.

## unsafe-ledger-after.jsonl

- Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/132-relation-descriptor-field-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/132-relation-descriptor-field-boundary > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/unsafe-ledger-generate.log 2>&1`
- Result: generated packet-local unsafe ledger for current `src/` state.

## unsafe-ledger-check.log

- Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/132-relation-descriptor-field-boundary/artifacts/unsafe-ledger-after.jsonl > reviews/task-50/132-relation-descriptor-field-boundary/artifacts/unsafe-ledger-check.log 2>&1`
- Result: `ledger covers 1578 current unsafe rows`.
