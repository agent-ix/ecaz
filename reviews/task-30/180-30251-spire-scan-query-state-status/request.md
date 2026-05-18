---
id: 30251
title: SPIRE Scan Query State Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 7b6650cc
---

# Review Request: SPIRE Scan Query State Status

## Summary

This docs-only checkpoint updates Task 30 after scan opaque state began storing
a validated query object.

- Notes that scan state has a non-empty, finite, non-zero query object ready for
  future live `ScanKey` parsing.
- Keeps live `ScanKey` parsing, relation-backed snapshot/object loading, heap
  rerank callback implementation, and PQ-FastScan scorer binding open.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the scan-path status makes the query-state boundary clear.
- Whether it avoids implying live `amrescan` parsing has landed.

## Validation

- `git diff --check`
- `git diff --cached --check`
