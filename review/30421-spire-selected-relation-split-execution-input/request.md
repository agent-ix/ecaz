# Review Request: SPIRE Selected Relation Split Execution Input

## Summary

Task 30 SPIRE Phase 2 now has a relation split execution-input helper that
consumes the selected publish-lock plan directly.

Changes:
- Add `build_relation_selected_scheduled_split_replacement_execution_input`.
- Keep the chosen split decision, PID plan, and publish plan bundled until
  relation split execution-input construction.
- Cover successful selected split planning and merge-plan rejection.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_selected_scheduled_split_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. Split centroid training and
routed leaf-input production remain live scheduler responsibilities.
