# Review Request: SPIRE Local Scheduled Replacement Execution Draft

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: local-store dry-run execution helper for scheduled replacements.

## Summary

- Added `SpireLocalScheduledReplacementExecutionInput`.
- Added `build_local_scheduled_replacement_epoch_draft`.
- The helper writes decision-bound replacement objects, validates the written
  placement output against the scheduled PID plan, then builds the scheduled
  replacement epoch draft.
- Added focused tests for successful local execution draft assembly and
  replacement-child/PID-plan order drift rejection.
- Updated the Task 30 Phase 2 checklist to record this local execution bridge.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test local_scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This is a local-store dry-run path. It does not write relation tuples or
  advance root/control.
- No measurement claims are made in this packet.
