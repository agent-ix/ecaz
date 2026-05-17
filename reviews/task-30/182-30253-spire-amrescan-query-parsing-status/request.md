---
id: 30253
title: SPIRE amrescan Query Parsing Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 2846a87f
---

# Review Request: SPIRE amrescan Query Parsing Status

## Summary

This docs-only checkpoint updates Task 30 after live `amrescan` query parsing
landed.

- Notes that `amrescan` validates scan shape, decodes the ORDER BY query, stores
  it in opaque state, and stops at relation-backed snapshot loading.
- Keeps snapshot/object loading, heap rerank callback implementation, and
  PQ-FastScan scorer binding open.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the scan-path status accurately separates query parsing from
  relation-backed scan execution.
- Whether the remaining unimplemented scan boundaries are explicit enough.

## Validation

- `git diff --check`
- `git diff --cached --check`
