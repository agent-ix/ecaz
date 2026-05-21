---
task: 50
packet: 125-spire-timestamp-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: e2486f93
code_commit: e2486f93
status: ready-for-review
---

# Review Request: SPIRE Timestamp Boundary

## Summary

This slice centralizes PostgreSQL `GetCurrentTimestamp()` access behind
`crate::storage::time::current_timestamp_micros()`.

It removes direct timestamp unsafe blocks from:

- `src/am/ec_spire/build/drafts.rs`
- `src/am/ec_spire/coordinator/snapshots.rs`

The remaining timestamp unsafe is isolated in `src/storage/time.rs`, returning
the backend timestamp by value without retaining PostgreSQL-owned pointers or
memory.

## Unsafe Burndown

- Previous packet count: `1632` unsafe blocks across `121` files.
- This packet count: `1630` unsafe blocks across `122` files.
- Net change: `-2` direct unsafe blocks.

The file count increases by one because the residual irreducible timestamp FFI
boundary now lives in a dedicated storage helper instead of being repeated at
SPIRE call sites.

## Validation

Artifacts are under `reviews/task-50/125-spire-timestamp-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1630 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1630`, `files 122`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
