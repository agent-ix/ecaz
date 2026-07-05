# Review Request: SPIRE Scheduled Execution Decision Validation

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement execution assembly
against the checked scheduler decision before local or relation object writes.

Changes:

- Extend the shared scheduled replacement execution-input validation path to
  take the scheduler decision.
- Reject invalid decision shape, replacement-parent PID drift, and replacement
  child-count drift before object writes begin.
- Thread the decision through local and relation publish-plan input builders and
  publish-plan validators.
- Add focused coverage for parent-PID and child-count drift.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure execution assembly
validation and does not add PostgreSQL callback coverage.
