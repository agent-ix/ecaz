# Review Request: SPIRE Relation Scheduled Merge Execution Parts

## Summary

Task 30 SPIRE Phase 2 now has a pure relation execution-parts builder for
scheduled merge decisions.

Changes:
- Compose scheduled merge routing parts with folded replacement leaf rows.
- Validate replacement leaf input coverage against the replacement child.
- Carry publish/retention timestamps and replacement object versions into
  `SpireRelationScheduledReplacementExecutionParts`.
- Cover successful composition plus missing leaf rows and reused-PID drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_scheduled_merge_replacement_execution_parts --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This leaves the live scheduler with data loading, publish-plan binding, and
relation publish orchestration rather than merge object-shape assembly.
No measurement claims; no PG callback coverage.
