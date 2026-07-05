# Review Request: SPIRE Local Scheduled Publish Plan Validation

## Summary

Task 30 SPIRE Phase 2 now validates local scheduled replacement execution
inputs against the checked publish plan before local dry-run object writes.

Changes:

- Add `validate_local_scheduled_replacement_execution_publish_plan`.
- Require `build_local_scheduled_replacement_epoch_draft` callers to pass the
  checked publish plan and validate execution input drift before writing local
  replacement objects.
- Extend focused local execution coverage for publish-plan drift.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test local_scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is local dry-run assembly
only and does not add PostgreSQL callback coverage.
