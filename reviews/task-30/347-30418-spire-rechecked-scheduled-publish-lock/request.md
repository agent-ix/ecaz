# Review Request: SPIRE Rechecked Scheduled Publish Lock

## Summary

Task 30 SPIRE Phase 2 now has a pure publish-lock helper that rechecks the
selected scheduler decision before allocating replacement PIDs and deriving the
publish plan.

Changes:
- Add `plan_rechecked_scheduled_replacement_publish_lock`.
- Reuse the existing selector/recheck contract before PID allocation.
- Preserve the caller's PID allocator cursor when the decision is no longer
  recommended under the lock-time leaf snapshot.
- Cover successful lock-step planning and changed-decision rejection.
- Update the Phase 2 checklist.

## Validation

- `cargo test rechecked_scheduled_replacement_publish_lock --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This prepares the live
scheduler path to bind selection recheck, PID allocation, and publish-plan
derivation under one pure seam.
