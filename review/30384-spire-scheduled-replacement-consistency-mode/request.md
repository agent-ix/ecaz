# Review Request: SPIRE Scheduled Replacement Consistency Mode

## Summary

Task 30 SPIRE Phase 2 now tightens scheduled replacement epoch drafts so
publication cannot silently switch consistency modes while replacing partition
objects.

Changes:

- Require `build_scheduled_replacement_epoch_draft_from_object_placements` input
  consistency mode to match the active epoch snapshot consistency mode.
- Extend focused scheduled replacement draft rejection coverage for
  consistency-mode drift.
- Update the Task 30 Phase 2 checklist to record the new guard.

## Validation

- `cargo test scheduled_replacement_epoch_draft --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. Scheduled replacement publication still
preserves the active epoch consistency mode; live relation execution remains the
larger integration step.
