# SPIRE Update Mechanics Review Follow-Up

## Checkpoint

- Code commit: `8c277456`
  (`Clarify SPIRE update mechanics plan`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30335`

## Summary

This checkpoint tightens the Phase 2 update-mechanics design note based on
review feedback.

The note now records:

- single-level split/merge routing rewrite cost scales with `nlists` because
  the flat root is the parent routing object
- replacement PIDs come from the same root/control PID allocator cursor as
  inserts, which is why split/merge/rebalance share the publish lock
- rebalance may reuse a PID only when the parent-routing centroid remains
  byte-equal
- retained prior epochs keep old PIDs queryable through their own placement
  directories until retention and active-query rules allow cleanup
- the concrete scheduler is undecided, with manual SQL, VACUUM-time hook, and
  background worker listed as first implementation candidates
- cross-parent merge is a multi-parent coverage rewrite, not a rebalance

## Changed Files

- `plan/design/spire-update-mechanics.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`

Tests were not run because this is a documentation-only review follow-up.

## Notes

- This directly responds to the packet `30335` review.
- Split/merge implementation and scheduling remain future work.
