---
id: 30160
title: SPIRE Task Plan
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: ba3854e1
---
# Review Request: SPIRE Task Plan

## Summary

This planning checkpoint turns ADR-049 into an implementation task.

The checkpoint:

- adds `plan/tasks/30-spire-ivf-foundation.md`
- stages SPIRE work as Phase 0 design reconciliation, single-level SPIRE-IVF foundation, update mechanics, recursion, boundary replication, top-level graph routing, and product-scale measurement
- calls out the assignment-storage design note as the first implementation blocker
- updates `plan/tasks/README.md` with Task 30

## Files To Review

- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/tasks/README.md`

## Validation

- `git diff --cached --check`
- No code tests run. This is a planning-only checkpoint under the repository checkpoint policy.

## Reviewer Focus

1. Does Task 30 decompose ADR-049 into the right implementation phases?
2. Is Phase 0 correctly scoped around landed-IVF reuse and assignment-storage design?
3. Are recursion, boundary replication, and top-level graph routing kept out of the first implementation slice?
4. Are dependencies and validation expectations realistic?
