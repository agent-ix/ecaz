---
id: 30240
title: SPIRE Single-Level Scan Option Plan
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 9bb7a6ba
---

# Review Request: SPIRE Single-Level Scan Option Plan

## Summary

This checkpoint adds a helper that resolves SPIRE reloptions plus session
overrides into a concrete single-level scan plan.

- Adds `SpireSingleLevelScanPlan`.
- Resolves effective `nprobe` from leaf count, relation option, and
  `ec_spire.nprobe`.
- Resolves effective rerank width from relation option and
  `ec_spire.rerank_width`.
- Maps storage-format options to assignment payload formats.
- Derives the approximate candidate limit used before exact rerank:
  - positive rerank width => `Some(width)`
  - zero rerank width => `None`, meaning full frontier
- Adds validation for invalid manually constructed negative option values.

## Non-Goals

- No AM scan callback execution.
- No heap exact-rerank implementation.
- No relation-backed snapshot loading.
- No change to the helper-level routed scan API yet.

## Review Focus

- Whether `SpireSingleLevelScanPlan` has the right fields for the future scan
  descriptor.
- Whether using rerank width as the pre-rerank candidate limit is the right
  default for SPIRE before SQL LIMIT is visible to the AM.
- Whether zero rerank width should mean full frontier or disabled rerank when
  live callback wiring begins.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 155 passed, 0 failed
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
