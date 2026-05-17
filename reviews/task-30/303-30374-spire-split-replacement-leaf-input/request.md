# Review Request: SPIRE Split Replacement Leaf Input

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure helper for validating split replacement leaf-object inputs after
  row routing.

## Summary

- Added `build_split_replacement_leaf_object_inputs`.
- The helper validates split decision shape, requires fresh replacement PIDs,
  checks that routed leaf inputs exactly cover the planned replacement PIDs,
  orders the returned inputs by PID-plan order, and reuses the replacement
  leaf-object input validator for row normalization and duplicate `vec_id`
  rejection.
- Added focused tests for unordered valid inputs and invalid input count /
  duplicate `vec_id` rejection.
- Updated the Task 30 Phase 2 checklist to record this split execution
  preparation slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test split_replacement_leaf_inputs --lib`
- `git diff --check`

## Notes

- This does not train split centroids or route rows by score; it validates the
  post-routing object-input shape that live split execution will consume.
- No measurement claims are made in this packet.
