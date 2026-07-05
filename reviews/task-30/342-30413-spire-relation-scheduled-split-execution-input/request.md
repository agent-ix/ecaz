# Review Request: SPIRE Relation Scheduled Split Execution Input

## Summary

Task 30 SPIRE Phase 2 now has a pure helper that composes split execution
parts into the final relation scheduled replacement execution input using the
checked publish plan.

Changes:
- Add `build_relation_scheduled_split_replacement_execution_input`.
- Reuse the split execution-parts helper and publish-plan input builder.
- Cover successful publish-plan binding plus next-PID and leaf-input drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_scheduled_split_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

Split centroid training and routed leaf-input production remain live scheduler
responsibilities. This helper validates the relation execution input once those
inputs are available.
No measurement claims; no PG callback coverage.
