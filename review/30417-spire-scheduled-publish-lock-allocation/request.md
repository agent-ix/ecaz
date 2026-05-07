# Review Request: SPIRE Scheduled Publish-Lock Allocation

## Summary

Task 30 SPIRE Phase 2 now has a pure publish-lock planning helper that allocates
scheduled replacement PIDs and derives the checked publish plan as one output.

Changes:
- Add `SpireScheduledReplacementPublishLockPlan`.
- Add `plan_scheduled_replacement_publish_lock`.
- Use a scratch PID allocator so publish-plan drift does not advance the
  caller's allocator cursor.
- Cover successful split PID/publish planning and no-advance behavior when the
  root/control epoch is stale.
- Update the Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_publish_lock --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This prepares the live
scheduler path to allocate PIDs and publish metadata under one lock-step seam.
