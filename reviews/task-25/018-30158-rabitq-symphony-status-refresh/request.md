---
id: 30158
title: RaBitQ and Symphony Status Refresh
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: 5068a651
---
# Review Request: RaBitQ and Symphony Status Refresh

## Summary

This planning/ADR checkpoint aligns the RaBitQ and Symphony docs with the current roadmap decision.

The checkpoint:

- marks Task 25 as landed for the first-class RaBitQ quantizer and IVF `rabitq` storage/profile support
- marks Task 27 Symphony as shelved indefinitely
- marks ADR-045 SymphonyQG as `SHELVED`
- marks ADR-031 RaBitQ binary prefilter as `SUPERSEDED` by the landed quantizer/profile surface
- updates the task and ADR indexes plus the master spec deferral list

## Files To Review

- `plan/tasks/25-rabitq-quantizer.md`
- `plan/tasks/27-symphony-access-method.md`
- `plan/tasks/README.md`
- `spec/adr/ADR-031-rabitq-binary-prefilter.md`
- `spec/adr/ADR-045-symphonyqg-quantized-graph-access-method.md`
- `spec/adr/index.md`
- `spec/spec.md`

## Validation

- `git diff --cached --check`
- No code tests run. This is a docs/planning-only checkpoint under the repository checkpoint policy.

## Reviewer Focus

1. Does Task 25 accurately separate landed RaBitQ/IVF work from shelved Symphony work?
2. Is Task 27 clearly non-active while preserving useful historical design context?
3. Do ADR-031 and ADR-045 use the right lifecycle statuses?
4. Are the indexes and master spec deferral list consistent with the roadmap decision?
