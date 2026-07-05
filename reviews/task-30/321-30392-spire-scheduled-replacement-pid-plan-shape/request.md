# Review Request: SPIRE Scheduled Replacement PID Plan Shape

## Summary

Task 30 SPIRE Phase 2 now tightens scheduled replacement publish-plan PID plan
shape validation.

Changes:

- Reject PID plans whose replacement PID count does not match the scheduler
  decision replacement count.
- Reject duplicate replacement PIDs before publish planning succeeds.
- Extend focused publish-plan rejection coverage.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_publish_plan --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure publish-lock plan
validation and does not add PostgreSQL callback coverage.
