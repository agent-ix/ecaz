# Review Request: SPIRE Selected Execution Snapshot Validator

## Summary

Task 30 SPIRE Phase 2 now has a selected-plan snapshot validator for scheduled
replacement execution.

Changes:
- Add `validate_selected_scheduled_replacement_execution_snapshot`.
- Keep selected decision and publish plan bundled during active snapshot epoch
  and consistency-mode drift checks.
- Cover successful validation plus epoch and consistency drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test selected_scheduled_replacement_execution_snapshot_validator --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure selected-plan
validation slice.
