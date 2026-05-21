---
head_sha: cb86e1f19ed4fd362be6ee2afab8b416a246f801
task_bucket: reviews/task-50
packet: reviews/task-50/130-ivf-insert-relation-oid-boundary
timestamp: 2026-05-20T18:27:39-07:00
lane: unsafe-burndown
fixture: static-analysis
storage_format: n/a
rerank_mode: n/a
isolation: n/a - static compile and ledger validation only
---

# Artifact Manifest

## code-diff.patch

- Command: `git show --format=fuller --stat --patch cb86e1f1 > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/code-diff.patch`
- Result: records code commit `cb86e1f19ed4fd362be6ee2afab8b416a246f801`.
- Key lines: routes IVF insert bootstrap lock OID read through `crate::storage::relation::relation_oid`.

## git-diff-check.log

- Command: `git diff --check cb86e1f1^ cb86e1f1 > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/git-diff-check.log`
- Result: pass; log is empty.

## src-unsafe-block-count-after.log

- Command: `make unsafe-block-count > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/src-unsafe-block-count-after.log`
- Result: after this slice, `src/` has `1585` unsafe blocks across `121` files.
- Key count change: previous packet 129 recorded `1586` unsafe blocks across `121` files.

## count-summary.md

- Command: `make unsafe-block-count | awk '{s+=$1; f+=1} END {print "unsafe_blocks " s; print "files " f}' > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/count-summary.md`
- Result:
  - `unsafe_blocks 1585`
  - `files 121`

## cargo-check-pg18-bench.log

- Command: `cargo check --all-targets --no-default-features --features pg18,bench > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/cargo-check-pg18-bench.log 2>&1`
- Result: pass.
- Note: cargo still reports the known pre-existing `src/am/mod.rs` SPIRE DML unused-import warning.

## unsafe-ledger-after.jsonl

- Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/130-ivf-insert-relation-oid-boundary > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/unsafe-ledger-generate.log 2>&1`
- Result: generated packet-local unsafe ledger for current `src/` state.

## unsafe-ledger-check.log

- Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/unsafe-ledger-after.jsonl > reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/unsafe-ledger-check.log 2>&1`
- Result: `ledger covers 1585 current unsafe rows`.
