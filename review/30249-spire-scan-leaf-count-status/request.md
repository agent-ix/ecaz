---
id: 30249
title: SPIRE Scan Leaf Count Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: ac88b366
---

# Review Request: SPIRE Scan Leaf Count Status

## Summary

This docs-only checkpoint updates Task 30 after root routing metadata gained a
helper for scan-plan leaf counts.

- Notes that `amrescan` can derive leaf count from a loaded published snapshot's
  root routing object in a future wiring slice.
- Keeps query parsing, relation-backed snapshot/object loading, heap rerank
  callback implementation, and PQ-FastScan scorer binding open.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the scan-path status now captures the leaf-count bridge without
  implying `amrescan` persistence loading has landed.
- Whether the remaining scan gaps are still specific enough.

## Validation

- `git diff --check`
- `git diff --cached --check`
