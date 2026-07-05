---
id: 30237
title: SPIRE Quantized Rerank Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 2e0f9abd
---

# Review Request: SPIRE Quantized Rerank Status

## Summary

This docs-only checkpoint updates Task 30 after helper-level quantized routed
scan and exact-rerank composition landed.

- Notes that scan helpers now compose route, quantized assignment scoring,
  `vec_id` dedupe, candidate limiting, and exact-rerank callback application.
- Keeps AM callback execution and heap exact-score implementation open.
- Keeps PQ-FastScan scorer binding deferred until grouped-PQ model metadata
  exists.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether Task 30 now draws the right line between helper-level scan behavior
  and still-open AM callback execution.
- Whether the remaining heap rerank and PQ-FastScan items are visible enough
  for subsequent slices.

## Validation

- `git diff --check`
- `git diff --cached --check`
