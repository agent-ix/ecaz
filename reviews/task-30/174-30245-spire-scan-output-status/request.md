---
id: 30245
title: SPIRE Scan Output Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 8454def3
---

# Review Request: SPIRE Scan Output Status

## Summary

This docs-only checkpoint updates Task 30 after the scan-output bridge landed.

- Notes that routed helper candidates can now map to heap TID plus ORDER BY
  score output for future `amgettuple` wiring.
- Keeps live AM callback execution, heap-row rerank callback implementation,
  and PQ-FastScan scorer binding explicitly open.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the Task 30 scan-path status captures the new helper boundary without
  implying relation-backed scan execution has landed.
- Whether the remaining open scan gaps are still clear.

## Validation

- `git diff --check`
- `git diff --cached --check`
