# Review Request: SPIRE Scheduled Routing PID Cursor

## Summary

Task 30 SPIRE Phase 2 now validates scheduled routing replacement PID cursor
advancement before producing replacement routing children.

Changes:

- Reject PID plans whose `next_pid` does not advance past every replacement PID
  in `build_scheduled_routing_replacement_children`.
- Extend focused routing-child rejection coverage.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_routing_replacement_children --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure routing input
validation and does not add PostgreSQL callback coverage.
