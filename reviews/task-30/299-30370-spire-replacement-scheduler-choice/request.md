# Review Request: SPIRE Replacement Scheduler Choice

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure scheduler-choice helper over existing leaf snapshot diagnostics.

## Summary

- Added `SpireLeafReplacementScheduleMode`,
  `SpireLeafReplacementScheduleDecision`, and
  `choose_leaf_replacement_schedule`.
- The selector validates that diagnostic rows belong to one active epoch,
  rejects ambiguous split+merge recommendations, prefers the largest split
  candidate, and otherwise selects the sparsest same-parent merge pair.
- The decision includes the active epoch, parent PID, affected leaf PIDs, and
  replacement leaf count so the future live scheduler can re-load and re-check
  the selected PIDs under the publish lock before writing objects.
- Updated the Task 30 Phase 2 checklist to record the scheduler-choice slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_scheduler --lib`
- `git diff --check`

## Notes

- This is advisory selection only. Live split/merge execution, centroid
  training/recomputation, and relation object publication from the selected
  decision remain open.
- No measurement claims are made in this packet.
- PQ-FastScan payloads, remote placement, and replica behavior remain deferred.
