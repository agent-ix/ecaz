---
id: 30248
title: SPIRE Snapshot Leaf Count Helper
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 29d08892
---

# Review Request: SPIRE Snapshot Leaf Count Helper

## Summary

This checkpoint adds a helper-level bridge from published root routing metadata
to the leaf count needed by scan option resolution.

- Adds `count_snapshot_single_level_leaf_pids()` to derive leaf count from the
  root routing object's child PID list.
- Covers the empty-leaf case by building a three-leaf root where the middle
  leaf has no assignments.
- Keeps relation-backed snapshot/object loading unwired.

## Non-Goals

- No persistence reads from PostgreSQL relations.
- No `amrescan` query parsing or plan resolution wiring.
- No build or placement format change.

## Review Focus

- Whether root children are the right source of truth for the single-level scan
  plan's `leaf_count`.
- Whether counting empty leaves through root routing metadata is the expected
  behavior for nprobe clamping.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 161 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
