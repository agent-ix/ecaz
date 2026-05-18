# SPIRE Replacement Placement Directory

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE Phase 2 update mechanics
- Scope: Pure placement-directory planning helper for replacement epochs

## Summary

This checkpoint adds a pure helper that plans the new active placement
directory after replacement leaf planning and parent-routing rewrite.

The helper:

- requires the replacement epoch to be newer than the base active epoch
- carries unaffected active placements forward with the new epoch
- drops the replaced parent routing placement from the new active directory
- drops affected old leaf placements from the new active directory
- drops active delta placements attached to affected leaves
- inserts the rewritten parent routing placement and replacement leaf
  placements
- validates replacement placements are available, have the replacement epoch,
  and do not duplicate leaf PIDs

This encodes the retention contract needed by split/merge: old PIDs disappear
from the new active placement directory but remain queryable through retained
prior epoch placement directories until cleanup eligibility allows old objects
to be reclaimed.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_placement --lib`
- `git diff --check`

## Notes

- No live scheduler, SQL entrypoint, or relation-backed publish path is added.
- No measurement claims.
- PQ-FastScan populated support, remote placement, and replicas remain
  deferred.
