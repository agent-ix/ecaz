# Review Request: SPIRE Selected Local Draft Preflight

## Summary

Task 30 SPIRE Phase 2 now has selected-plan preflight validation for local
scheduled replacement draft assembly.

Changes:
- Add `validate_local_selected_scheduled_replacement_draft_inputs`.
- Wire the selected local draft builder through the preflight before object
  writes.
- Keep selected decision, PID plan, and publish plan bundled through local
  execution-input and active snapshot validation.
- Cover successful preflight plus execution-input and snapshot drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_replacement_draft_inputs --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure local dry-run
preflight slice.
