# Review Request: SPIRE Scheduled Execution Publish Timestamp

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement execution inputs for
a publish timestamp before object writes.

Changes:

- Extend shared scheduled replacement execution validation to reject
  `published_at_micros <= 0`.
- Cover the local execution validator rejection for missing publish timestamps.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure execution assembly
validation and does not add PostgreSQL callback coverage.
