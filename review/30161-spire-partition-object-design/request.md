---
id: 30161
title: SPIRE Partition Object Design
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: 8c32a192
---
# Review Request: SPIRE Partition Object Design

## Summary

This planning checkpoint revises ADR-049 and Task 30 after reading the SPIRE
paper and discussing the intended scale path.

The checkpoint:

- changes ADR-049 from a simple logical assignment-table decision to a
  PID-addressed partition-object storage decision
- clarifies that SPIRE partitions are index-internal objects, not PostgreSQL
  declarative table partitions
- introduces bounded local partition stores for local multi-NVMe placement
- preserves the future placement extension from
  `pid -> local_store_id` to `pid -> node_id -> local_store_id`
- adds epoch/version requirements for compatible root metadata, hierarchy
  metadata, placement metadata, and partition objects
- starts formal requirements coverage with `US-017` and `FR-038`

## Files To Review

- `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md`
- `plan/tasks/30-spire-ivf-foundation.md`
- `spec/usecase/US-017-build-and-scale-spire.md`
- `spec/functional/FR-038-spire-partition-object-storage.md`
- `spec/spec.md`
- `spec/adr/index.md`
- `plan/tasks/README.md`

## Validation

- `git diff --check`
- No code tests run. This is a planning/spec-only checkpoint under the
  repository checkpoint policy.

## Reviewer Focus

1. Does ADR-049 now match the SPIRE paper's partition-object and PID-placement
   model closely enough for Phase 0 implementation planning?
2. Is the local multi-NVMe placement stage correctly separated from the later
   multi-machine placement stage?
3. Are `vec_id`, local heap TID, PID, placement, and epoch/version concerns
   captured at the right level of specificity?
4. Does `FR-038` provide enough initial functional coverage without
   overcommitting implementation details before Phase 0?
