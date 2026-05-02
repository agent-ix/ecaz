---
id: 30241
title: SPIRE Scan Option Plan Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: c681c53e
---

# Review Request: SPIRE Scan Option Plan Status

## Summary

This docs-only checkpoint updates Task 30 after scan option resolution gained a
single-level scan plan helper.

- Notes that SPIRE option settings now resolve to effective `nprobe`, assignment
  payload format, rerank width, and pre-rerank candidate limit.
- Keeps live build/scan callback consumption explicitly open.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether the Task 30 status now makes the option-plumbing boundary clear.
- Whether the remaining live callback consumption language is specific enough.

## Validation

- `git diff --check`
- `git diff --cached --check`
