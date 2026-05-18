# Review Request: SPIRE Scheduled Replacement Publish Draft

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure helper for binding a scheduler decision to replacement epoch
  draft assembly after replacement objects and placement entries are available.

## Summary

- Added `SpireScheduledReplacementEpochObjectPlacementInput`.
- Added `build_scheduled_replacement_epoch_draft_from_object_placements`.
- The helper validates scheduler decision shape, rejects active
  snapshot/decision epoch mismatches, rejects replacement leaf-placement count
  mismatches, then delegates placement-directory, object-manifest, and
  root/control validation to the existing replacement publish-draft helper.
- Added focused tests for successful decision-bound publish-draft assembly,
  snapshot epoch mismatch rejection, and replacement placement count rejection.
- Updated the Task 30 Phase 2 checklist to record this scheduler execution
  preparation slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_replacement_epoch_draft --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This does not perform live relation object writes or root/control publication.
  It prepares the decision-bound draft assembly that live scheduler execution
  can call after relation writes succeed under the publish lock.
- No measurement claims are made in this packet.
