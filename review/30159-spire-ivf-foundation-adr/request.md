---
id: 30159
title: SPIRE IVF Foundation ADR
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: e4e5f61d
---
# Review Request: SPIRE IVF Foundation ADR

## Summary

This docs checkpoint adds ADR-049 for the SPIRE staging decision.

The checkpoint:

- adds `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md`
- records the decision to build single-level IVF first, then layer SPIRE on top
- records the schema choice to store partition assignments as `(vec_id, partition_id)` rows
- records that SPIRE should stay modular inside one extension without speculative pluggable strategy abstractions
- adds ADR-049 to `spec/adr/index.md`

## Files To Review

- `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md`
- `spec/adr/index.md`

## Validation

- `git diff --cached --check`
- No code tests run. This is an ADR-only checkpoint under the repository checkpoint policy.

## Reviewer Focus

1. Is `ADR-049` the right next ADR number and filename?
2. Does the ADR preserve the intended SPIRE staging and assignment-table decisions?
3. Is the line between single-level IVF foundation and later SPIRE recursion/boundary replication clear?
4. Is the ADR index placement/status correct?
