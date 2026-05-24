---
task: 50
packet: 131-active-snapshot-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: f7e072c5af4b9e27aae2a7f955b7413b891621c9
code_commit: f7e072c5af4b9e27aae2a7f955b7413b891621c9
status: ready-for-review
---

# Review Request: Active Snapshot Boundary

## Summary

This slice centralizes backend active-snapshot reads behind
`crate::storage::snapshot_guard::active_snapshot()`.

It migrates non-dirty call sites in:

- `src/am/ec_diskann/scan_state.rs`
- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_spire/coordinator/lifecycle.rs`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`

The remaining direct `GetActiveSnapshot` outside the storage helper is in
`src/am/ec_ivf/scan.rs`, left untouched because that file already has unrelated
local edits in the worktree.

## Unsafe Burndown

- Previous packet count: `1585` unsafe blocks across `121` files.
- This packet count: `1581` unsafe blocks across `121` files.
- Net change: `-4` direct unsafe blocks.

## Validation

Artifacts are under `reviews/task-50/131-active-snapshot-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1581 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1581`, `files 121`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
