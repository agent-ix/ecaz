# Review Request: SPIRE Scheduler Duplicate Row Guard

## Summary

Task 30 SPIRE Phase 2 now rejects duplicate leaf snapshot rows before choosing
or rechecking a replacement schedule.

Changes:
- Add duplicate `leaf_pid` rejection to scheduler row validation.
- Extend scheduler-choice rejection coverage.
- Update the Phase 2 checklist.

## Validation

- `cargo test replacement_scheduler --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This is a selector hygiene guard for the advisory scheduler surface.
No measurement claims; no PG callback coverage.
