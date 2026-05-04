# Review Request: SPIRE Scheduled Routing Replacement Children

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure helper for turning scheduled replacement PIDs plus replacement
  centroids into routing replacement children.

## Summary

- Added `build_scheduled_routing_replacement_children`.
- The helper validates scheduler decision shape, requires fresh replacement
  PIDs, checks replacement PID count against the decision, checks centroid
  count against the PID plan, rejects duplicate/zero replacement PIDs, and
  rejects empty or non-finite centroids.
- The returned routing replacement children preserve PID-plan order so the
  existing routing rewrite helper can splice them into the parent routing
  object and validate exact parent dimensions.
- Updated the Task 30 Phase 2 checklist to record this scheduler execution
  preparation slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_routing_replacement_children --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This does not train or recompute replacement centroids; it validates the
  scheduled routing-child shape once the live scheduler supplies those
  centroids.
- No measurement claims are made in this packet.
