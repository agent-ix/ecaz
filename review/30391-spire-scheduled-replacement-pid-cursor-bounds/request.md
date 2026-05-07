# Review Request: SPIRE Scheduled Replacement PID Cursor Bounds

## Summary

Task 30 SPIRE Phase 2 now tightens scheduled replacement publish-plan PID cursor
validation.

Changes:

- Reject replacement PIDs that are behind the root/control `next_pid` cursor.
- Reject PID plans whose final `next_pid` does not advance past every
  replacement PID.
- Extend focused publish-plan rejection coverage.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_publish_plan --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure publish-lock plan
validation and does not add PostgreSQL callback coverage.
