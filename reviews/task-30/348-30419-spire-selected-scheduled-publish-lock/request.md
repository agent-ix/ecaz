# Review Request: SPIRE Selected Scheduled Publish Lock

## Summary

Task 30 SPIRE Phase 2 now has a pure selector wrapper that chooses the
lock-time scheduled replacement decision and returns it with the checked
publish-lock plan.

Changes:
- Add `SpireSelectedScheduledReplacementPublishLockPlan`.
- Add `choose_scheduled_replacement_publish_lock_plan`.
- Return `None` without advancing the PID allocator when no split/merge
  replacement is recommended.
- Cover selected split planning and no-decision/no-allocation behavior.
- Update the Phase 2 checklist.

## Validation

- `cargo test selected_scheduled_replacement_publish_lock --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is the pure scheduler
selection seam before constructing merge/split execution inputs.
