---
task: 50
packet: 128-spire-relation-oid-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: ac2150c2fe5e3ef0b88d249ffa010ef2ee6b6cdc
code_commit: ac2150c2fe5e3ef0b88d249ffa010ef2ee6b6cdc
status: ready-for-review
---

# Review Request: SPIRE Relation OID Boundary

## Summary

This slice removes direct SPIRE relation OID unsafe reads by routing them
through `crate::storage::relation::relation_oid()`.

Touched surfaces include:

- publish locking
- active snapshot diagnostics
- relation object store construction
- local store relation planning
- insert local-store config creation
- remote candidate libpq planning
- remote endpoint identity diagnostics

The remaining SPIRE direct `(*relation).rd_id` unsafe outside the centralized
storage helper is in `src/am/ec_spire/dml_frontdoor/mod.rs`; that belongs to a
larger DML context slice.

## Unsafe Burndown

- Previous packet count: `1596` unsafe blocks across `123` files.
- This packet count: `1587` unsafe blocks across `121` files.
- Net change: `-9` direct unsafe blocks and `-2` files with unsafe blocks.

## Validation

Artifacts are under `reviews/task-50/128-spire-relation-oid-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1587 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1587`, `files 121`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
