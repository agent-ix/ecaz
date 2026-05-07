---
id: 30247
title: SPIRE Scan Opaque Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 53da1c2c
---

# Review Request: SPIRE Scan Opaque Status

## Summary

This docs-only checkpoint updates Task 30 after SPIRE gained scan opaque
lifecycle state.

- Notes that scan callbacks now allocate opaque state and can drain a populated
  candidate cursor through `amgettuple`.
- Keeps `amrescan` query parsing, relation-backed snapshot/object loading,
  heap rerank callback implementation, and PQ-FastScan scorer binding open.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the status accurately describes callback-state progress without
  implying relation-backed scan execution has landed.
- Whether the remaining callback and persistence gaps are explicit enough.

## Validation

- `git diff --check`
- `git diff --cached --check`
