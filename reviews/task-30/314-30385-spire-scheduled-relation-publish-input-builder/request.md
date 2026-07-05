# Review Request: SPIRE Scheduled Relation Publish Input Builder

## Summary

Task 30 SPIRE Phase 2 now has a pure bridge from the checked scheduled
replacement publish plan into the relation scheduled replacement execution
input.

Changes:

- Add `SpireRelationScheduledReplacementExecutionParts` for caller-supplied
  rewritten routing and replacement leaf objects.
- Add
  `build_relation_scheduled_replacement_execution_input_from_publish_plan`,
  which preserves the planned epoch, active consistency mode, and local vector
  cursor from `SpireScheduledReplacementPublishPlan`.
- Reject reused PID plans, PID cursor drift, and replacement-child PID order
  mismatches before relation writes begin.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test relation_scheduled_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The helper is a pure live-scheduler
assembly guard; it does not add PostgreSQL callback coverage.
