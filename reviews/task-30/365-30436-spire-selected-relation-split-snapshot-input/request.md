# Review Request: SPIRE Selected Relation Split Snapshot Input

## Summary

Task 30 SPIRE Phase 2 now has a relation split execution-input helper that
loads the selected parent routing object from the active snapshot before input
construction.

Changes:
- Add `build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot`.
- Load the selected parent routing object through the selected publish-lock
  plan.
- Preserve caller-trained split centroids and routed replacement leaf rows as
  scheduler responsibilities.
- Compose selected-plan relation split execution input without relation I/O.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_selected_scheduled_split_replacement_execution_input_from_snapshot --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure material-loading
helper for later live scheduler relation publication wiring.
