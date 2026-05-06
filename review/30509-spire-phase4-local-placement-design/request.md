# Review Request: SPIRE Phase 4 Local Multi-Store Placement Design

## Checkpoint

- Code commit: `247b25d2`
  (`Document SPIRE local multi-store placement design`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local multi-store placement design

## Summary

Task 30 Phase 4 now has a durable design checkpoint in
`plan/design/spire-local-multistore-placement.md`.

The design records:

- a bounded relation-local `local_store_count` / `local_store_tablespaces`
  configuration surface;
- legacy/default single-store compatibility where store 0 remains the
  root/control index relation;
- dedicated AM-owned partition-store relations for multi-store indexes;
- root/control ownership of the active store set through a versioned store
  generation and store descriptors;
- placement-entry validation against the active store set, including
  `local_store_id`, `store_relid`, object TID, object bytes, and placement
  state;
- deterministic PID hash placement for leaf and routing objects, with deltas
  colocated with the parent leaf store;
- open and lock ordering across root/control and store relations;
- strict and degraded behavior when one local store is stale, skipped, or
  unavailable;
- a store-grouped scan fetch boundary that keeps candidate scoring close to
  partition-object bytes while deferring any benchmark-backed parallel
  multi-NVMe claim;
- placement and scan-placement diagnostic rows that report store identity,
  tablespace identity, object counts, bytes, candidate rows, scanned PID
  counts, and skipped placement state.

The Task 30 tracker now marks the design checkpoint complete and leaves the
implementation items open.

## Files

- `plan/design/spire-local-multistore-placement.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review whether the design is tight enough for the next implementation
slices:

- local store reloptions and root/control metadata codecs;
- store relation create/open/discovery helpers;
- deterministic hash placement planning;
- routing object, leaf object, and delta object write routing;
- store-grouped scan reads and placement diagnostics.

In particular, check whether the design makes the single-store compatibility
boundary explicit enough and whether the lock ordering, publish atomicity, and
degraded-store semantics are sufficient before relation-helper code lands.

## Validation

- `git diff --check`
- `git diff --cached --check`

Tests were not run because this is a documentation-only checkpoint.

## Notes

No measurement claims. The design explicitly defers one-store versus
multi-store benchmark claims until packet-local artifacts exist from a host
with multiple physical NVMe devices.
