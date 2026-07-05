# Review Request: SPIRE Scheduled Routing Object Version

## Summary

Task 30 SPIRE Phase 2 now validates scheduled replacement parent-routing object
versions before rewriting.

Changes:

- Reject `object_version == 0` in
  `rewrite_scheduled_replacement_parent_routing`.
- Extend focused scheduled routing rewrite rejection coverage.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_routing_rewrite --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure routing rewrite
validation and does not add PostgreSQL callback coverage.
