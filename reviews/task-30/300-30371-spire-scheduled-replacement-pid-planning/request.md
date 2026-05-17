# Review Request: SPIRE Scheduled Replacement PID Planning

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: convert advisory scheduler decisions into replacement PID allocation
  plans.

## Summary

- Added `plan_scheduled_leaf_replacement_pids`.
- The helper validates scheduler-decision shape before touching the allocator
  cursor, maps split/merge decisions to the existing replacement PID planner,
  and returns the replacement PID plan plus next root/control PID cursor.
- Added focused tests for split/merge allocation from scheduler decisions and
  malformed-decision rejection without allocator advancement.
- Updated the Task 30 Phase 2 checklist to record this slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_replacement_pid_plan --lib`
- `git diff --check`

## Notes

- This checkpoint still stops before live scheduler execution, centroid
  training/recomputation, and relation replacement object publication from a
  selected decision.
- No measurement claims are made in this packet.
