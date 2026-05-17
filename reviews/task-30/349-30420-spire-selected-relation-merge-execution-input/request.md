# Review Request: SPIRE Selected Relation Merge Execution Input

## Summary

Task 30 SPIRE Phase 2 now has a relation merge execution-input helper that
consumes the selected publish-lock plan directly.

Changes:
- Add `build_relation_selected_scheduled_merge_replacement_execution_input`.
- Keep the chosen decision, PID plan, and publish plan bundled until relation
  merge execution-input construction.
- Cover successful selected merge planning and split-plan rejection.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_selected_scheduled_merge_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure relation
scheduler composition slice for merge replacements.
