# SPIRE Single-Store Placement Complete

## Checkpoint

- Code commit: `1b1e14cf`
  (`Mark SPIRE single-store placement complete`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Task-plan closeout for strict local single-store PID placement

## Summary

This checkpoint marks the Phase 1 single-store placement task complete in the
Task 30 plan.

The already-landed implementation now covers the intended Phase 1 placement
shape:

- Populated builds persist one root routing object, one V2 leaf object per
  centroid, durable placement-entry tuples, manifest bundles, and active
  root/control state.
- Insert and delete-delta epochs carry forward existing placement entries and
  append delta placements under strict local single-store placement.
- Vacuum compaction rewrites affected base leaves as replacement V2 objects and
  publishes a replacement active placement directory without active delta
  placements.
- Relation-storage diagnostics continue tracking no-longer-active tuple debt;
  physical tuple reclamation remains a separate vacuum/cleanup item.

## Changed Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not run because this checkpoint only updates task-plan status for
already-landed implementation work.

## Notes

- This does not close PQ-FastScan scan binding, physical old-epoch cleanup,
  SQL `VACUUM` end-to-end coverage, insert batching, or concurrency stress.
- Local multi-store and remote placement remain later phases.
