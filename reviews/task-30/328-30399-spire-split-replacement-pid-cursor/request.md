# Review Request: SPIRE Split Replacement PID Cursor

## Summary

Task 30 SPIRE Phase 2 now validates split replacement leaf-input PID cursor
advancement before ordering caller-routed leaf inputs.

Changes:

- Reject split PID plans whose `next_pid` does not advance past every
  replacement PID.
- Extend focused split replacement leaf-input rejection coverage.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test split_replacement_leaf_inputs --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure split helper
validation and does not add PostgreSQL callback coverage.
