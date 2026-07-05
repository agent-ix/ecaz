# Review Request: SPIRE Selected Merge Snapshot Draft

## Summary

Task 30 SPIRE Phase 2 now has a local dry-run merge helper that loads the
selected snapshot materials before building the scheduled replacement draft.

Changes:
- Add `build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot`.
- Load the selected parent routing object through the selected publish-lock
  plan.
- Collect folded affected-leaf rows through the selected publish-lock plan.
- Compose the loaded materials with selected-plan merge draft assembly.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot --lib`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This keeps merge material
loading local and pure before relation-backed publication wiring.
