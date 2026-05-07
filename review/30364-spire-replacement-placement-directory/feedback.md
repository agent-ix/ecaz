# 30364 SPIRE Replacement Placement Directory — feedback

## What landed

`plan_replacement_epoch_placement_directory` carries unaffected active
placements forward with the new epoch, drops the replaced parent routing
placement, drops affected old leaf placements, drops active delta
placements attached to those affected leaves, and inserts the rewritten
parent + replacement leaf placements.

## Correctness

- New epoch must be strictly newer than the snapshot's epoch
  (`new_epoch <= snapshot.epoch_manifest().epoch` rejection, line 949).
- Validates that both replacement parent and replacement leaf placements
  carry the new epoch, the right PID, and that leaf placement PIDs are
  unique and never collide with the parent PID.
- Snapshot must already be valid + delta base placements must be Available
  (`validate_delta_base_snapshot_placements_available`) before the
  directory is rewritten — keeps the strict-publish invariant.
- Old PIDs intentionally remain queryable only through retained prior
  placement directories (per design §"Deltas and Visibility"); the new
  active directory simply doesn't reference them.

## Status

Lands cleanly. Old/retired placements get their natural lifecycle through
manifest retention rather than a separate cleanup path here.
