# Review Request: SPIRE Local Replacement Object Writes

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: local object-store helper for writing a replacement parent routing
  object plus replacement V2 leaf objects.

## Summary

- Added `SpireReplacementObjectPlacements` and
  `write_local_replacement_objects` in `ec_spire::update`.
- The helper validates replacement leaf object inputs, rejects invalid epochs
  and leaf object versions, writes the replacement root/internal routing object,
  writes replacement V2 leaves with the routing parent PID, and returns
  placements ordered by replacement routing children.
- Added focused unit coverage that writes a rewritten root and unordered leaf
  inputs, then verifies stored routing children, published-epoch backrefs, leaf
  parent PIDs, and placement ordering.
- Updated the Task 30 Phase 2 checklist to record the local writer slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test local_replacement_object_writer --lib`
- `git diff --check`

## Notes

- This remains local-store helper coverage only. Live relation-backed
  replacement object writes, scheduler execution, and root/control publication
  wiring remain open.
- No measurement claims are made in this packet.
- PQ-FastScan payloads, remote placement, and replica behavior remain deferred.
