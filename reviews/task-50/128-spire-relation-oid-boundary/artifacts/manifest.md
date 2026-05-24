---
head_sha: ac2150c2fe5e3ef0b88d249ffa010ef2ee6b6cdc
task_bucket: reviews/task-50
packet: reviews/task-50/128-spire-relation-oid-boundary
timestamp: 2026-05-20T18:21:21-07:00
lane: unsafe-burndown
fixture: static-analysis
storage_format: n/a
rerank_mode: n/a
isolation: n/a - static compile and ledger validation only
---

# Artifact Manifest

## code-diff.patch

- Command: `git show --format=fuller --stat --patch ac2150c2 > reviews/task-50/128-spire-relation-oid-boundary/artifacts/code-diff.patch`
- Result: records code commit `ac2150c2fe5e3ef0b88d249ffa010ef2ee6b6cdc`.
- Key lines: routes SPIRE relation OID reads through `crate::storage::relation::relation_oid`.

## git-diff-check.log

- Command: `git diff --check ac2150c2^ ac2150c2 > reviews/task-50/128-spire-relation-oid-boundary/artifacts/git-diff-check.log`
- Result: pass; log is empty.

## src-unsafe-block-count-after.log

- Command: `make unsafe-block-count > reviews/task-50/128-spire-relation-oid-boundary/artifacts/src-unsafe-block-count-after.log`
- Result: after this slice, `src/` has `1587` unsafe blocks across `121` files.
- Key count change: previous packet 127 recorded `1596` unsafe blocks across `123` files.

## count-summary.md

- Command: `make unsafe-block-count | awk '{s+=$1; f+=1} END {print "unsafe_blocks " s; print "files " f}' > reviews/task-50/128-spire-relation-oid-boundary/artifacts/count-summary.md`
- Result:
  - `unsafe_blocks 1587`
  - `files 121`

## cargo-check-pg18-bench.log

- Command: `cargo check --all-targets --no-default-features --features pg18,bench > reviews/task-50/128-spire-relation-oid-boundary/artifacts/cargo-check-pg18-bench.log 2>&1`
- Result: pass.
- Note: cargo still reports the known pre-existing `src/am/mod.rs` SPIRE DML unused-import warning.

## unsafe-ledger-after.jsonl

- Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/128-spire-relation-oid-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/128-spire-relation-oid-boundary > reviews/task-50/128-spire-relation-oid-boundary/artifacts/unsafe-ledger-generate.log 2>&1`
- Result: generated packet-local unsafe ledger for current `src/` state.

## unsafe-ledger-check.log

- Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/128-spire-relation-oid-boundary/artifacts/unsafe-ledger-after.jsonl > reviews/task-50/128-spire-relation-oid-boundary/artifacts/unsafe-ledger-check.log 2>&1`
- Result: `ledger covers 1587 current unsafe rows`.
