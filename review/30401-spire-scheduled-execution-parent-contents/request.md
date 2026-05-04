# Review Request: SPIRE Scheduled Execution Parent Contents

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement execution inputs
against the rewritten parent routing contents before object writes.

Changes:
- Reject execution inputs whose replacement parent does not contain every
  replacement child PID.
- Reject execution inputs whose replacement parent still contains affected leaf
  PIDs.
- Add focused rejection coverage for unrewritten and stale-leaf parent routing
  input.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet responds to the cross-cutting feedback in
`review/30388-spire-local-scheduled-publish-plan-validation/feedback.md`.
No measurement claims; no PG callback coverage.
