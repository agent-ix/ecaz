---
id: 30162
title: SPIRE Phase 0 Partition-Object Storage
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: d3c141a2
---
# Review Request: SPIRE Phase 0 Partition-Object Storage

## Summary

This Phase 0 checkpoint decides the concrete SPIRE partition-object storage
shape before persistence implementation begins.

The checkpoint records:

- PostgreSQL-managed relation-backed partition stores as the default storage
  shape, starting with the `ec_spire` index relation as `local_store_id = 0`
- PID-addressed immutable partition objects with per-partition object versions
  referenced by epoch manifests
- logical leaf assignment rows carrying stable `vec_id`, local heap locator,
  encoded scoring payload, and primary/boundary/tombstone/delta/stale flags
- index-local `vec_id` allocation for Phase 1, encoded as discriminator plus
  local sequence, instead of deriving identity from heap TID
- heap TID semantics as local row locator only, with HOT handled by PostgreSQL's
  index contract and non-HOT updates modeled as delete-old plus insert-new
- placement map shape `pid -> local_store_id -> object`, preserving the later
  `pid -> node_id -> local_store_id -> object` extension
- strict local single-store default, configurable degraded mode for local
  multi-store and remote deployments
- replacement-epoch plus delta-object lifecycle for inserts, deletes, vacuum,
  split/merge, rebalance, failed publish handling, retention, and cleanup
- reuse/factor plan for landed `ec_ivf` training, quantizer, scan, and admin
  components while keeping the `ec_ivf` on-disk format unchanged
- Phase 1 exposure as opt-in `ec_spire` with
  `ecvector_spire_ip_ops` and `tqvector_spire_ip_ops`

## Files To Review

- `plan/design/spire-phase0-partition-object-storage.md`
- `plan/tasks/30-spire-ivf-foundation.md`
- `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md`
- `spec/functional/FR-038-spire-partition-object-storage.md`
- `spec/functional/FR-039-spire-local-nvme-placement.md`
- `spec/functional/FR-041-spire-epoch-consistency.md`
- `spec/functional/FR-043-spire-update-split-merge-lifecycle.md`
- `spec/spec.md`

## Validation

- `git diff --cached --check`
- No code tests run. This is a docs-only Phase 0 checkpoint under the
  repository checkpoint policy.

## Reviewer Focus

1. Is the Phase 0 storage shape concrete enough to start SPIRE persistence
   without collapsing into speculative raw-file or PostgreSQL table-partition
   designs?
2. Are `pid`, `vec_id`, heap locator, parent/child PID, and assignment flag
   semantics explicit enough for Phase 1 code and later remote merge?
3. Does the index-local `vec_id` choice correctly avoid heap-TID identity
   hazards while preserving a global-ID epoch rewrite path?
4. Are HOT, non-HOT UPDATE, delete, vacuum, failed publish, retention, and
   cleanup semantics compatible with PostgreSQL index AM behavior?
5. Is the `ec_ivf` reuse plan appropriately conservative, reusing training and
   scoring machinery without forcing the existing IVF format into SPIRE?
