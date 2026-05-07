# Review Request: SPIRE Selected Relation Merge Snapshot Input

## Summary

Task 30 SPIRE Phase 2 now has a relation merge execution-input helper that
loads selected materials from the active snapshot before relation input
construction.

Changes:
- Add `build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot`.
- Load the selected parent routing object through the selected publish-lock
  plan.
- Collect folded affected-leaf rows through the selected publish-lock plan.
- Compose selected-plan relation merge execution input without relation I/O.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_selected_scheduled_merge_replacement_execution_input_from_snapshot --lib`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure material-loading
helper for later live scheduler relation publication wiring.
