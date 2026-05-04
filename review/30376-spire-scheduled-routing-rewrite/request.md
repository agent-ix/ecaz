# Review Request: SPIRE Scheduled Routing Rewrite

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure helper for binding a scheduler decision to the parent routing
  object rewrite.

## Summary

- Added `rewrite_scheduled_replacement_parent_routing`.
- The helper validates scheduler decision shape, rejects loading a parent
  routing object whose PID differs from the scheduler decision, and rejects
  replacement-child count mismatches before delegating to the existing routing
  rewrite helper.
- The existing rewrite helper still owns affected-child membership checks,
  replacement PID collision checks, centroid dimensionality checks, and root vs
  internal object reconstruction.
- Added focused tests for split rewrite, merge rewrite, wrong-parent rejection,
  and replacement-child count rejection.
- Updated the Task 30 Phase 2 checklist to record this scheduler execution
  preparation slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_routing_rewrite --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This remains helper-level wiring. Live scheduler execution still needs to
  load the parent routing object, recompute/validate centroids, write relation
  objects, and publish the replacement epoch under the publish lock.
- No measurement claims are made in this packet.
