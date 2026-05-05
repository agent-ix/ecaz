# Review Request: SPIRE Scheduled Replacement Object Versions

## Summary

Task 30 SPIRE Phase 2 now has a pure object-version planner for scheduled
replacement execution.

Changes:
- Add `SpireScheduledReplacementObjectVersionPlan` for replacement parent and
  replacement leaf object versions.
- Derive replacement parent version as the active parent object's successor.
- Derive replacement leaf version as the maximum successor across affected
  active leaf object versions.
- Reject zero, overflow, duplicate, and missing affected-leaf version inputs.
- Cover split, merge/max-version, and missing-leaf cases with unit tests.

## Validation

- `cargo test scheduled_replacement_object_version_plan --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This avoids hardcoded object-version inputs when the live scheduler starts
building relation replacement execution inputs.
No measurement claims.
