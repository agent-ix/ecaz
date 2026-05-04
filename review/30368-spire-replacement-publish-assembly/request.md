# Review Request: SPIRE Replacement Publish Assembly

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: assemble a replacement epoch draft from already-written replacement
  object placements and placement-write evidence.

## Summary

- Added `SpireReplacementEpochObjectPlacementInput` and
  `build_replacement_epoch_draft_from_object_placements`.
- The helper plans the replacement epoch placement directory from the active
  snapshot, replacement parent placement, and replacement leaf placements, then
  builds the validated replacement epoch draft using the caller-supplied
  placement-write evidence and allocator cursors.
- Added focused local-store coverage that writes replacement objects, assembles
  the replacement epoch draft, verifies affected old leaf/delta removal,
  verifies replacement manifest placement TIDs, and checks root/control cursor
  propagation.
- Updated the Task 30 Phase 2 checklist to record the publish assembly helper.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_epoch_draft_from_object_placements --lib`
- `git diff --check`

## Notes

- This checkpoint stops before live scheduler execution and root/control
  relation publication for split/merge epochs.
- No measurement claims are made in this packet.
- PQ-FastScan payloads, remote placement, and replica behavior remain deferred.
