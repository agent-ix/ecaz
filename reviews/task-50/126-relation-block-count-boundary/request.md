---
task: 50
packet: 126-relation-block-count-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: 030751f886f449d8dae5f8ee651d808c1ca34312
code_commit: 030751f886f449d8dae5f8ee651d808c1ca34312
status: ready-for-review
---

# Review Request: Relation Block Count Boundary

## Summary

This broad slice centralizes PostgreSQL main-fork block count reads behind
`crate::storage::relation::main_fork_block_count()`.

It removes repeated direct `RelationGetNumberOfBlocksInFork(... MAIN_FORKNUM)`
unsafe blocks from SPIRE, IVF admin/vacuum, DiskANN, HNSW, and test helper
call sites. The pre-existing planner helper now delegates to the same storage
helper.

The only remaining direct `RelationGetNumberOfBlocksInFork` call sites are:

- `src/storage/relation.rs`: the centralized residual FFI boundary.
- `src/am/ec_ivf/page.rs`: left untouched because that file already has
  unrelated local edits in the worktree.

## Unsafe Burndown

- Previous packet count: `1630` unsafe blocks across `122` files.
- This packet count: `1607` unsafe blocks across `123` files.
- Net change: `-23` direct unsafe blocks.

The file count increases by one because the residual relation block-count FFI
boundary now lives in a storage helper instead of being repeated through AM
code.

## Validation

Artifacts are under `reviews/task-50/126-relation-block-count-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1607 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1607`, `files 123`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
