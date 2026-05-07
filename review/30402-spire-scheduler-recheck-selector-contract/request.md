# Review Request: SPIRE Scheduler Recheck Selector Contract

## Summary

Task 30 SPIRE Phase 2 now documents that the scheduler selector and
publish-lock recheck must move in lockstep.

Changes:
- Add an inline contract comment on
  `recheck_leaf_replacement_schedule_decision`.
- Update the Phase 2 checklist to record the selector ranking/tie-break
  coupling.

## Validation

- `cargo fmt --check`
- `git diff --check`

Tests were skipped under the checkpoint policy because this is a comment and
task-doc-only clarification.

## Notes

This responds to the minor concern in
`review/30372-spire-replacement-scheduler-recheck/feedback.md`.
No measurement claims; no PG callback coverage.
