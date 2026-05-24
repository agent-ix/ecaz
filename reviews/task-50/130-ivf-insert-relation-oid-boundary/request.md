---
task: 50
packet: 130-ivf-insert-relation-oid-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: cb86e1f19ed4fd362be6ee2afab8b416a246f801
code_commit: cb86e1f19ed4fd362be6ee2afab8b416a246f801
status: ready-for-review
---

# Review Request: IVF Insert Relation OID Boundary

## Summary

This slice removes one IVF insert direct relation OID unsafe read.

`lock_empty_bootstrap_relation()` now uses
`crate::storage::relation::relation_oid(index_relation)` instead of directly
dereferencing `(*index_relation).rd_id`.

## Unsafe Burndown

- Previous packet count: `1586` unsafe blocks across `121` files.
- This packet count: `1585` unsafe blocks across `121` files.
- Net change: `-1` direct unsafe block.

## Validation

Artifacts are under `reviews/task-50/130-ivf-insert-relation-oid-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1585 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1585`, `files 121`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
