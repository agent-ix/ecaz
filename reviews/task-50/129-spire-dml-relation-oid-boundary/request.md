---
task: 50
packet: 129-spire-dml-relation-oid-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: 8ae776c0d05853a57f4a3842f2b4b44fc913dafc
code_commit: 8ae776c0d05853a57f4a3842f2b4b44fc913dafc
status: ready-for-review
---

# Review Request: SPIRE DML Relation OID Boundary

## Summary

This slice removes the remaining SPIRE direct relation OID unsafe read outside
the centralized storage relation helper.

`dml_frontdoor_relation_context_catalog_for_open_heap()` now uses
`crate::storage::relation::relation_oid(heap_relation)` instead of directly
dereferencing `(*heap_relation).rd_id`.

## Unsafe Burndown

- Previous packet count: `1587` unsafe blocks across `121` files.
- This packet count: `1586` unsafe blocks across `121` files.
- Net change: `-1` direct unsafe block.

## Validation

Artifacts are under `reviews/task-50/129-spire-dml-relation-oid-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1586 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1586`, `files 121`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
