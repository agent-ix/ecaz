# Review Request: SPIRE Selected Local Merge Draft

## Summary

Task 30 SPIRE Phase 2 now has a local dry-run merge helper that consumes the
selected publish-lock plan and builds the scheduled replacement draft in one
step.

Changes:
- Add `build_local_selected_scheduled_merge_replacement_epoch_draft`.
- Compose selected-plan merge execution-input construction with selected local
  draft assembly.
- Cover successful merge draft construction and split-plan rejection.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_merge_replacement_epoch_draft --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. Merge leaf-row collection and
selected-plan acquisition remain live scheduler responsibilities.
