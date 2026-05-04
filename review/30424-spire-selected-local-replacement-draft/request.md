# Review Request: SPIRE Selected Local Replacement Draft

## Summary

Task 30 SPIRE Phase 2 now has a local dry-run draft builder that consumes the
selected publish-lock plan directly.

Changes:
- Add `build_local_selected_scheduled_replacement_epoch_draft`.
- Keep selected decision, PID plan, and publish plan bundled through local
  replacement object writes and draft assembly.
- Cover successful selected split draft assembly and snapshot epoch drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_replacement_epoch_draft --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure local dry-run
composition slice for the selected scheduler plan.
