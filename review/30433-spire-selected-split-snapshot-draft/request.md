# Review Request: SPIRE Selected Split Snapshot Draft

## Summary

Task 30 SPIRE Phase 2 now has a local dry-run split helper that loads the
selected parent routing object from the active snapshot before draft assembly.

Changes:
- Add `build_local_selected_scheduled_split_replacement_epoch_draft_from_snapshot`.
- Load the selected parent routing object through the selected publish-lock
  plan before split execution-input construction.
- Preserve caller-trained centroids and routed leaf inputs as live scheduler
  responsibilities.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_split_replacement_epoch_draft_from_snapshot --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. Split centroid training and
routing remain outside this helper.
