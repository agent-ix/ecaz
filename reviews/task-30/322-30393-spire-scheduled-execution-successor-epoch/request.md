# Review Request: SPIRE Scheduled Execution Successor Epoch

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement execution publish
plans for immediate-successor epochs before object writes.

Changes:

- Extend shared scheduled replacement execution validation to require
  `publish_plan.epoch == decision.active_epoch + 1`.
- Cover the local execution validator rejection for successor-epoch drift.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure execution assembly
validation and does not add PostgreSQL callback coverage.
