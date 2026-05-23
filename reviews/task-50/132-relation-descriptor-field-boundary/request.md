---
task: 50
packet: 132-relation-descriptor-field-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: 961190cd675cdb7408b6b17af4c30daad3699cef
code_commit: 961190cd675cdb7408b6b17af4c30daad3699cef
status: ready-for-review
---

# Review Request: Relation Descriptor Field Boundary

## Summary

This slice centralizes two copied relation descriptor reads:

- `crate::storage::relation::relation_reltuples()`
- `crate::storage::relation::relation_tablespace()`

It removes direct `rd_rel` unsafe reads from SPIRE local-store config planning,
SPIRE debug local-store config setup, IVF admin diagnostics, and the common
planner-cost reltuples helper.

## Unsafe Burndown

- Previous packet count: `1581` unsafe blocks across `121` files.
- This packet count: `1578` unsafe blocks across `120` files.
- Net change: `-3` direct unsafe blocks and `-1` file with unsafe blocks.

## Validation

Artifacts are under `reviews/task-50/132-relation-descriptor-field-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1578 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1578`, `files 120`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
