# Review Request: SPIRE Recursive Route Coordinator

## Summary

Task 30 Phase 3 scan routing now has a pure recursive coordinator over
already-loaded routing objects.

Changes:

- Add `route_recursive_routing_objects_to_leaf_pids(...)`.
- Reuse the level-local `route_routing_object_to_child_pids(...)` primitive at
  each routing level.
- Preserve the existing single-level root path by delegating root level 1 to
  `route_root_object_to_leaf_pids(...)`.
- Validate internal routing child presence, kind, parent PID, and level before
  descending.
- Return selected leaf PIDs once the coordinator reaches level-1 routing
  parents.
- Add focused coverage for successful root-to-internal-to-leaf routing, missing
  internal child failure, and wrong child-level failure.
- Update the Task 30 Phase 3 level-local scan primitive note.

## Validation

- `cargo test route_recursive_routing_objects_to_leaf_pids -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this slice. The coordinator is pure and still
expects routing objects to be preloaded; relation-backed recursive hierarchy
loading and live scan integration remain open.
