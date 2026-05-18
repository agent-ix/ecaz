# Review Request: SPIRE Merge Replacement Leaf Input

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure helper for building the replacement leaf-object input for a
  scheduled merge.

## Summary

- Added `build_merge_replacement_leaf_object_input`.
- The helper validates that the scheduler decision is a merge, requires exactly
  one fresh replacement PID, combines folded rows from all selected affected
  leaves in decision order, and rejects missing, duplicate, or unselected base
  PID row groups.
- The output is validated through the existing replacement leaf-object input
  validator before it can be passed to the replacement object writer.
- Updated the Task 30 Phase 2 checklist to record this merge execution
  preparation slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test merge_replacement_leaf_input --lib`
- `git diff --check`

## Notes

- This is still helper-level coverage. Live scheduler execution still needs to
  compute the replacement centroid, rewrite routing, write relation objects,
  and publish the replacement epoch.
- No measurement claims are made in this packet.
