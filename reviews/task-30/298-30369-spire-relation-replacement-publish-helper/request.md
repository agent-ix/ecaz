# Review Request: SPIRE Relation Replacement Publish Helper

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: relation publication wrapper for a replacement epoch after
  replacement objects have already been written.

## Summary

- Added `SpireRelationReplacementEpochObjectPlacementInput` and
  `publish_relation_replacement_epoch_from_object_placements`.
- The wrapper plans the replacement placement directory from the active
  snapshot and replacement object placements, writes placement-directory rows to
  the index relation, builds the validated replacement epoch draft, retires the
  matching previous epoch manifest, writes the new manifest bundle, and advances
  root/control through the existing publish coordinator.
- Added a guard that rejects a previous epoch manifest that does not match the
  base snapshot before relation placement rows are written.
- Updated the Task 30 Phase 2 checklist to record the relation replacement
  publish helper.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_epoch_draft_from_object_placements --lib`
- `git diff --check`

## Notes

- This checkpoint still stops before live split/merge scheduler execution.
- The new relation wrapper is compile-covered by the focused unit target; the
  runtime test remains local-store/pure assembly coverage.
- No measurement claims are made in this packet.
