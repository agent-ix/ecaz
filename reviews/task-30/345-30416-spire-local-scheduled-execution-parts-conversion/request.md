# Review Request: SPIRE Local Scheduled Execution Parts Conversion

## Summary

Task 30 SPIRE Phase 2 local scheduled merge/split helpers now share one
relation-to-local execution-parts conversion helper.

Changes:
- Add `local_scheduled_replacement_execution_parts_from_relation_parts`.
- Route local scheduled merge and split parts builders through the shared
  conversion helper.
- Keep placement-write evidence preservation at the local boundary.

## Validation

- `cargo test local_scheduled_ --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure cleanup slice
that keeps local dry-run execution composition aligned between merge and split.
