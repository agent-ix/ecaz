---
id: 30231
title: SPIRE Quantizer Reuse Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 0735ba8b
---

# Review Request: SPIRE Quantizer Reuse Status

## Summary

This docs-only checkpoint updates Task 30 status after the SPIRE assignment
quantizer scorer landed.

- Notes that SPIRE assignment payload scoring now reuses TurboQuant and RaBitQ
  through a SPIRE-owned row scorer.
- Records that PQ-FastScan remains deferred until grouped-PQ model metadata is
  persisted.
- Updates the Phase 1 build and scan bullets to distinguish implemented
  assignment payload scoring from still-open relation-backed AM execution and
  heap rerank callback integration.

## Non-Goals

- No ADR or Phase 0 design-note decision change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether Task 30's status now draws the right line between implemented
  helper-level scoring and still-unwired AM callback execution.
- Whether the PQ-FastScan deferral wording is clear enough for the next
  grouped-PQ persistence slice.

## Validation

- `git diff --check`
- `git diff --cached --check`
