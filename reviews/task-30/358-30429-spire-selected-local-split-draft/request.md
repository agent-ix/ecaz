# Review Request: SPIRE Selected Local Split Draft

## Summary

Task 30 SPIRE Phase 2 now has a local dry-run split helper that consumes the
selected publish-lock plan and builds the scheduled replacement draft in one
step.

Changes:
- Add `build_local_selected_scheduled_split_replacement_epoch_draft`.
- Compose selected-plan split execution-input construction with selected local
  draft assembly.
- Cover successful split draft construction and merge-plan rejection.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_split_replacement_epoch_draft --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. Split centroid training and
routed leaf-input production remain live scheduler responsibilities.
