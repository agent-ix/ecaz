# Review Request: SPIRE Relation Scheduled Merge Execution Input

## Summary

Task 30 SPIRE Phase 2 now has a pure helper that composes scheduled merge
execution parts into the final relation scheduled replacement execution input
using the checked publish plan.

Changes:
- Add `build_relation_scheduled_merge_replacement_execution_input`.
- Reuse the merge execution-parts helper and the publish-plan input builder.
- Cover successful publish-plan binding plus next-PID and object-version drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_scheduled_merge_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This further reduces live merge scheduler invocation to loading snapshot data,
building the publish plan, and calling the existing relation publish wrapper.
No measurement claims; no PG callback coverage.
