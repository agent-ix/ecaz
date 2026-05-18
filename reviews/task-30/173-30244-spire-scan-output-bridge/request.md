---
id: 30244
title: SPIRE Scan Output Bridge
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 8a8bb192
---

# Review Request: SPIRE Scan Output Bridge

## Summary

This checkpoint adds the final helper-level output shape needed before SPIRE
scan callbacks can emit routed candidates.

- Adds `SpireScanOutput` as the heap TID plus ORDER BY score shape consumed by
  `amgettuple`.
- Adds cursor output conversion via `SpireScanCandidateCursor::next_output()`.
- Adds scan descriptor helpers for setting heap TID, setting ORDER BY score,
  and clearing exhausted ORDER BY output.
- Leaves live scan callback execution and persistence loading unwired.

## Non-Goals

- No relation-backed SPIRE object reads.
- No heap tuple fetch or exact heap-row rerank callback implementation.
- No AM callback behavior change.

## Review Focus

- Whether the output shape is the right narrow bridge between ranked candidates
  and future `amgettuple` emission.
- Whether the scan descriptor helpers match the existing `ec_ivf` callback
  behavior closely enough for the next callback-wiring slice.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 158 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
