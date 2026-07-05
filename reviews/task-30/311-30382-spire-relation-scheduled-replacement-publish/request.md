# Review Request: SPIRE Relation Scheduled Replacement Publish

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: relation-side scheduled replacement publish wrapper.

## Summary

- Added `SpireRelationScheduledReplacementExecutionInput`.
- Added `publish_relation_scheduled_replacement_epoch`.
- The helper writes relation-backed replacement objects, validates the written
  placements against the scheduled PID plan, writes the replacement placement
  directory, builds the scheduled replacement epoch draft, and publishes through
  the existing replacement epoch root/control publisher.
- Updated the Task 30 Phase 2 checklist to record this relation execution
  bridge.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This checkpoint compiles the relation wrapper and covers the shared generic
  object-write and draft-assembly behavior through existing local scheduled
  replacement tests. It does not yet wire a live scheduler callback.
- No measurement claims are made in this packet.
