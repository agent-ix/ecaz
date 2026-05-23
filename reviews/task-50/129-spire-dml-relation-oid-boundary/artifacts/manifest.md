---
head_sha: 8ae776c0d05853a57f4a3842f2b4b44fc913dafc
task_bucket: reviews/task-50
packet: reviews/task-50/129-spire-dml-relation-oid-boundary
timestamp: 2026-05-20T18:24:34-07:00
lane: unsafe-burndown
fixture: static-analysis
storage_format: n/a
rerank_mode: n/a
isolation: n/a - static compile and ledger validation only
---

# Artifact Manifest

## code-diff.patch

- Command: `git show --format=fuller --stat --patch 8ae776c0 > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/code-diff.patch`
- Result: records code commit `8ae776c0d05853a57f4a3842f2b4b44fc913dafc`.
- Key lines: routes SPIRE DML frontdoor heap relation OID read through `crate::storage::relation::relation_oid`.

## git-diff-check.log

- Command: `git diff --check 8ae776c0^ 8ae776c0 > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/git-diff-check.log`
- Result: pass; log is empty.

## src-unsafe-block-count-after.log

- Command: `make unsafe-block-count > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/src-unsafe-block-count-after.log`
- Result: after this slice, `src/` has `1586` unsafe blocks across `121` files.
- Key count change: previous packet 128 recorded `1587` unsafe blocks across `121` files.

## count-summary.md

- Command: `make unsafe-block-count | awk '{s+=$1; f+=1} END {print "unsafe_blocks " s; print "files " f}' > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/count-summary.md`
- Result:
  - `unsafe_blocks 1586`
  - `files 121`

## cargo-check-pg18-bench.log

- Command: `cargo check --all-targets --no-default-features --features pg18,bench > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/cargo-check-pg18-bench.log 2>&1`
- Result: pass.
- Note: cargo still reports the known pre-existing `src/am/mod.rs` SPIRE DML unused-import warning.

## unsafe-ledger-after.jsonl

- Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/129-spire-dml-relation-oid-boundary > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/unsafe-ledger-generate.log 2>&1`
- Result: generated packet-local unsafe ledger for current `src/` state.

## unsafe-ledger-check.log

- Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/unsafe-ledger-after.jsonl > reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/unsafe-ledger-check.log 2>&1`
- Result: `ledger covers 1586 current unsafe rows`.
