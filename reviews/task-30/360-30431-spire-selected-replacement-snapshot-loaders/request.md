# Review Request: SPIRE Selected Replacement Snapshot Loaders

## Summary

Task 30 SPIRE Phase 2 now has selected-plan wrappers for loading scheduler
replacement inputs from the active snapshot.

Changes:
- Add `load_selected_scheduled_replacement_parent_routing`.
- Add `collect_selected_scheduled_replacement_leaf_rows`.
- Validate active snapshot epoch and consistency mode through the selected
  publish-lock plan before parent/leaf-row loading.
- Update the Phase 2 checklist.

## Validation

- `cargo test selected_scheduled_replacement_ --lib`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. These helpers are pure local
seams for live scheduler material loading before merge/split execution input
construction.
