---
id: 30161
title: SPIRE Partition Object Design
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: ec832599
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
- expands formal requirements coverage into local lifecycle, local multi-NVMe,
  distributed libpq query, epoch/rebalance, routing/search, update/split/merge,
  and storage/placement specs
- records graceful degradation as the preferred failure posture, with strict
  fail-closed available as a consistency mode
- records replicated partition objects as future work for read throughput and
  availability, not part of v1

Follow-up commit `ec832599` addresses the first review pass by:

- adding SPIRE traceability rows to `spec/tests.md` for StR-005, US-017..US-020,
  FR-038..FR-043, and planned TC-020..TC-025 coverage
- extending StR-005 relationships and success criteria for SPIRE planning
- rewriting FR-043-AC-1 as behavioral insert/delete visibility/failure
  acceptance criteria
- adding failed epoch publish and split/merge abort behavior to US-020,
  FR-041, and FR-043
- constraining `vec_id` width/uniqueness and recording the local/global
  discriminator expectation
- adding HOT/UPDATE heap-locator handling requirements to FR-038, FR-040,
  FR-043, and Phase 0 task work
- making local single-store strict mode the default while keeping degraded mode
  configurable for larger remote deployments
- adding the remote strict/degraded libpq coordinator AC and the Phase 1
  `ec_spire` opclass documentation deliverable

## Files To Review

- `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md`
- `plan/tasks/30-spire-ivf-foundation.md`
- `spec/usecase/US-017-build-and-scale-spire.md`
- `spec/usecase/US-018-operate-spire-local-nvme-stores.md`
- `spec/usecase/US-019-query-distributed-spire.md`
- `spec/usecase/US-020-manage-spire-epochs-and-rebalance.md`
- `spec/functional/FR-038-spire-partition-object-storage.md`
- `spec/functional/FR-039-spire-local-nvme-placement.md`
- `spec/functional/FR-040-spire-routing-and-search.md`
- `spec/functional/FR-041-spire-epoch-consistency.md`
- `spec/functional/FR-042-spire-distributed-libpq-coordinator.md`
- `spec/functional/FR-043-spire-update-split-merge-lifecycle.md`
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
4. Do the US files cover the standard local and remote lifecycle states clearly
   enough for planning implementation slices?
5. Do the FR files include the right schemas, diagrams, and process boundaries
   without overcommitting the exact physical table/relation layout before Phase
   0 measurement?
