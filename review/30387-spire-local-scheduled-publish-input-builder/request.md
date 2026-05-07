# Review Request: SPIRE Local Scheduled Publish Input Builder

## Summary

Task 30 SPIRE Phase 2 now has a local scheduled replacement execution-input
builder that consumes the checked publish plan.

Changes:

- Add `SpireLocalScheduledReplacementExecutionParts` for caller-supplied local
  dry-run pieces, including placement-write evidence.
- Add `build_local_scheduled_replacement_execution_input_from_publish_plan`,
  which carries the planned epoch, active consistency mode, and local vector
  cursor into `SpireLocalScheduledReplacementExecutionInput`.
- Share PID cursor, replacement-child order, and leaf-input validation with the
  relation execution-input builder.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test local_scheduled_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The helper is pure local execution
assembly and does not add PostgreSQL callback coverage.
