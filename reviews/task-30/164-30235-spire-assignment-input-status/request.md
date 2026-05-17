---
id: 30235
title: SPIRE Assignment Input Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 455003d8
---

# Review Request: SPIRE Assignment Input Status

## Summary

This docs-only checkpoint updates Task 30 after the source-vector assignment
input helper landed.

- Notes that the build path now has a source-vector helper that creates
  quantized leaf assignment inputs from heap locators plus source vectors.
- Keeps AM build/insert callback wiring explicitly open.
- Keeps PQ-FastScan blocked on persisted grouped-PQ model metadata.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether this status wording makes the helper-level progress clear without
  implying relation-backed build persistence is wired.
- Whether the build-path open items are still framed correctly.

## Validation

- `git diff --check`
- `git diff --cached --check`
