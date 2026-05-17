---
id: 30226
title: SPIRE Ranked Scan Cursor
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 20378188
---

# Review Request: SPIRE Ranked Scan Cursor

## Summary

This checkpoint adds a pure cursor over ranked SPIRE scan candidates.

- Adds `SpireScanCandidateCursor`.
- Owns a sorted `Vec<SpireScoredScanCandidate>` and an explicit next index.
- Emits each candidate at most once by reference.
- Exposes `remaining`, `is_exhausted`, and `reset` helpers for the eventual
  scan opaque state.
- Debug-asserts that constructor inputs are already sorted by the shared
  candidate comparator.

## Non-Goals

- No PostgreSQL scan descriptor allocation.
- No heap visibility or heap fetch integration.
- No order-by slot emission.
- No AM callback behavior.

## Review Focus

- Whether the cursor should enforce sorted input with a runtime error instead
  of a debug assertion.
- Whether the future AM callback should emit by reference as this helper does,
  or take ownership of candidates as it drains them.
- Whether reset should preserve allocation capacity once scan opaque state
  exists.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 134 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
