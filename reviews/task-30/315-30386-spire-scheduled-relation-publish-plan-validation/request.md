# Review Request: SPIRE Scheduled Relation Publish Plan Validation

## Summary

Task 30 SPIRE Phase 2 now validates relation scheduled replacement execution
inputs against the checked publish plan before relation object writes.

Changes:

- Add
  `validate_relation_scheduled_replacement_execution_publish_plan` for pure
  drift checks against `SpireScheduledReplacementPublishPlan`.
- Reuse the validator in the publish-plan input builder.
- Require `publish_relation_scheduled_replacement_epoch` callers to pass the
  checked publish plan and validate the execution input before writing relation
  objects.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test relation_scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change stays in pure helper and
relation wrapper assembly; no PostgreSQL callback test was added.
