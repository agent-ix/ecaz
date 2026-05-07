# SPIRE Update Mechanics Plan

## Checkpoint

- Code commit: `45ea0f25`
  (`Document SPIRE update mechanics plan`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 2 split/merge/rebalance planning over partition objects

## Summary

This checkpoint closes the Task 30 planning item that translates
LIRE/SPFresh-style update mechanics into SPIRE's PostgreSQL storage model.

The new design note records:

- published partition objects remain immutable
- split/merge allocate replacement PIDs when partition coverage changes
- rebalance may reuse PID with a new object version only when coverage is
  unchanged
- split/merge fold active insert/delete deltas into replacement V2 leaves
  before publication
- routing objects are rewritten in the replacement epoch when child coverage
  changes
- old object tuples become cleanup candidates after epoch retention
- the first implementation should use the existing insert/vacuum publish lock
  and epoch publication order

This is a planning checkpoint only. It does not implement split, merge, or
rebalance scheduling.

## Changed Files

- `plan/design/spire-update-mechanics.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not run because this is a documentation-only checkpoint.

## Notes

- Read-only split/merge threshold diagnostics were closed in packet `30332`.
- Actual split/merge execution remains future work.
