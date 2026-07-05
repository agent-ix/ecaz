# Review Request: SPIRE Scheduled Execution Active Snapshot

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement execution against the
active snapshot before local or relation object writes.

Changes:

- Add a shared pre-write active snapshot validation helper for scheduled
  replacement execution.
- Reject snapshot/decision active-epoch drift and publish-plan consistency-mode
  drift before object writes.
- Cover the local dry-run rejection for active snapshot consistency drift.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test local_scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure execution assembly
validation and does not add PostgreSQL callback coverage.
