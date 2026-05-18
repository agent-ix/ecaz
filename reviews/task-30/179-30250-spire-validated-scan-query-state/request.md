---
id: 30250
title: SPIRE Validated Scan Query State
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: c4a7aa9d
---

# Review Request: SPIRE Validated Scan Query State

## Summary

This checkpoint adds the validated query object that future SPIRE `amrescan`
query parsing will populate.

- Adds `SpireScanQuery` with dimension, finite-value, non-empty, and non-zero
  vector validation.
- Stores `SpireScanQuery` in scan opaque state instead of a raw vector.
- Keeps relation-backed snapshot/object loading and live `amrescan` parsing
  unwired.

## Non-Goals

- No PostgreSQL `ScanKey` datum parsing yet.
- No persistence reads or heap exact rerank implementation.
- No planner or cost-model change.

## Review Focus

- Whether `SpireScanQuery` captures the right validation before root-dimension
  validation happens during routing.
- Whether scan opaque should store the validated query object as optional state
  across rescans.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 163 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
