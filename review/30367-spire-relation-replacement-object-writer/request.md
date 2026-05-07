# Review Request: SPIRE Relation Replacement Object Writer

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: shared replacement-object write path for local and relation-backed
  object stores.

## Summary

- Added a private replacement-object writer abstraction used by the local
  helper and a new relation-backed wrapper.
- `write_local_replacement_objects` now delegates to the shared path, keeping
  the existing validation and replacement-child placement ordering.
- Added `write_relation_replacement_objects` so relation publish wiring can
  write the rewritten parent routing object and replacement V2 leaves through
  `SpireRelationObjectStore` and receive the same placement bundle shape.
- Updated the Task 30 Phase 2 checklist to record the relation-backed writer
  helper slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test local_replacement_object_writer --lib`
- `git diff --check`

## Notes

- This checkpoint adds the relation object-write helper but does not yet wire
  scheduler execution or root/control publication for split/merge epochs.
- No measurement claims are made in this packet.
- PQ-FastScan payloads, remote placement, and replica behavior remain deferred.
