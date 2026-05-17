# SPIRE Leaf Maintenance Triggers Complete

## Checkpoint

- Code commit: `70d9eb20`
  (`Mark SPIRE leaf maintenance triggers complete`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Task-plan closeout for read-only split/merge trigger thresholds

## Summary

This checkpoint marks the split and merge trigger-definition items complete in
the Task 30 plan.

The already-landed SQL leaf diagnostics expose:

- per-leaf base assignment counts
- delta object counts
- delta insert/delete counts
- effective assignment counts
- read-only split/merge threshold labels

The documented initial thresholds are:

- split candidate when effective assignments are at least
  `max(32, 4 * ceil(total_effective_assignments / active_leaf_count))`
- merge candidate when effective assignments are at or below
  `floor(ceil(total_effective_assignments / active_leaf_count) / 4)`

This checkpoint does not implement a split or merge scheduler.

## Changed Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not rerun for this documentation-only closeout. The leaf diagnostic
coverage was landed in earlier packets.

## Notes

- Actual split/merge mechanics remain future Phase 2 work.
- This only closes the read-only trigger-threshold definition items.
