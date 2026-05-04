# Review Request: SPIRE Replacement Scheduler Recheck

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: publish-lock recheck helper for advisory replacement scheduler
  decisions.

## Summary

- Added `recheck_leaf_replacement_schedule_decision`.
- The helper validates the expected decision shape, recomputes the advisory
  choice from fresh leaf snapshot rows, accepts a stable decision, and fails
  closed if the decision disappeared or changed before object writes.
- Added focused tests for stable recheck, changed decision rejection, and
  no-longer-recommended rejection.
- Updated the Task 30 Phase 2 checklist to record this slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_scheduler_recheck --lib`
- `git diff --check`

## Notes

- This is still pure helper coverage. Live split/merge execution must call it
  after reloading the active snapshot under the publish lock.
- No measurement claims are made in this packet.
