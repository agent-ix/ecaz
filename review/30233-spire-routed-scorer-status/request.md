---
id: 30233
title: SPIRE Routed Scorer Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 1040db71
---

# Review Request: SPIRE Routed Scorer Status

## Summary

This docs-only checkpoint updates Task 30 after the routed scan helper gained a
concrete quantized assignment scorer wrapper.

- Distinguishes the older injected scorer seam from the new routed helper that
  prepares SPIRE assignment payload scorers directly.
- Notes that TurboQuant and RaBitQ assignment payload scoring can now run over
  real encoded routed rows.
- Leaves heap rerank callback integration, AM callback execution, and
  PQ-FastScan scorer binding explicitly open.

## Non-Goals

- No ADR or Phase 0 design decision change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the Task 30 status now describes the scan helper boundary accurately.
- Whether the remaining open scan items are framed narrowly enough for the next
  implementation slices.

## Validation

- `git diff --check`
- `git diff --cached --check`
