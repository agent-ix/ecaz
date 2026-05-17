# Review Request: SPIRE Scheduled Replacement Publish Plan

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure publish-lock planning helper for scheduled replacements.

## Summary

- Added `SpireScheduledReplacementPublishPlan`.
- Added `plan_scheduled_replacement_publish_epoch`.
- The helper binds root/control active epoch and allocator cursors, the active
  epoch manifest, the checked scheduler decision, and the fresh PID plan into
  the immediate replacement publish plan.
- Added focused tests for successful planning, stale decision rejection, and PID
  cursor regression rejection.
- Updated the Task 30 Phase 2 checklist to record this publish-lock planning
  guardrail.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_replacement_publish_plan --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This is the pure planning step that should run after scheduler recheck and
  PID allocation under the relation publish lock, before relation object writes.
- No measurement claims are made in this packet.
