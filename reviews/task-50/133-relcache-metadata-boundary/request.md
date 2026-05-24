---
task: 50
packet: 133-relcache-metadata-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: c16fcbfbb52e2d003166d90c121bc15485b210ff
code_commit: c16fcbfbb52e2d003166d90c121bc15485b210ff
status: ready-for-review
---

# Review Request: Relcache Metadata Boundary

## Summary

This slice centralizes copied relcache metadata reads behind storage relation
helpers:

- `relation_name`
- `relation_kind`
- `relation_am_oid`
- `relation_namespace_owner_persistence`

It removes direct relcache unsafe reads from index validation in `src/lib.rs`,
DiskANN build relation naming, common explain AM lookup, and SPIRE local-store
relation planning.

## Unsafe Burndown

- Previous packet count: `1578` unsafe blocks across `120` files.
- This packet count: `1575` unsafe blocks across `120` files.
- Net change: `-3` direct unsafe blocks.

## Validation

Artifacts are under `reviews/task-50/133-relcache-metadata-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1575 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1575`, `files 120`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
