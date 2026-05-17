# Review Request: SPIRE Merge Replacement PID Cursor

## Summary

Task 30 SPIRE Phase 2 now validates merge replacement leaf-input PID cursor
advancement before building the merged replacement leaf input.

Changes:

- Reject merge PID plans whose `next_pid` does not advance past the single
  replacement PID.
- Extend focused merge replacement leaf-input rejection coverage.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test merge_replacement_leaf_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure merge helper
validation and does not add PostgreSQL callback coverage.
