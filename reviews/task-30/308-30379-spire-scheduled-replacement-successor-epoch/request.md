# Review Request: SPIRE Scheduled Replacement Successor Epoch

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: scheduled replacement publish-draft guardrail.

## Summary

- Tightened `build_scheduled_replacement_epoch_draft_from_object_placements`
  to require the replacement epoch to be exactly `decision.active_epoch + 1`.
- Added focused rejection coverage for skipped publish epochs.
- Updated the Task 30 Phase 2 checklist to record the immediate-successor
  publication requirement.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_replacement_epoch_draft --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This keeps scheduled replacement publication aligned with the existing insert
  and vacuum replacement paths, which publish the next active epoch under the
  publish lock.
- No measurement claims are made in this packet.
