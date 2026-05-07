# Review Request: SPIRE Scheduled Object Writer Successor Epoch

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement object writes for
the immediate successor epoch.

Changes:

- Require `write_local_scheduled_replacement_objects` and
  `write_relation_scheduled_replacement_objects` callers to use
  `decision.active_epoch + 1`.
- Add focused local object-writer rejection coverage for non-successor epochs.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test local_scheduled_replacement_object_writer --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure object-writer
validation and does not add PostgreSQL callback coverage.
