# Review Request: SPIRE Selected Split Input From Sources

## Summary

Task 30 SPIRE Phase 2 now has a selected-plan relation execution-input builder
for split replacement from fetched source vectors.

Changes:
- Add
  `build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot_sources`.
- Load the selected parent routing object and folded selected leaf rows from
  the active snapshot.
- Hydrate fetched source vectors, train/route split materialization, and feed
  the existing selected relation split execution-input validator.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This keeps the live unsafe
wrapper focused on fetching source vectors under the selected publish-lock
plan, then calling this checked builder.
