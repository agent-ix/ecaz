# Review Request: SPIRE Level-Local Route Primitive

## Summary

Task 30 Phase 3 scan routing now has a pure level-local route primitive.

Changes:

- Factor the existing root-only bounded route heap into
  `route_routing_object_to_child_pids(...)`.
- Allow the primitive to route either root or internal routing objects.
- Preserve the existing `route_root_object_to_leaf_pids(...)` root-kind guard
  for the current single-level scan path.
- Reuse the existing deterministic ordering: higher inner product, lower
  centroid ordinal, then lower child PID.
- Add focused coverage for internal-level routing and for the root wrapper
  still rejecting an internal parent.
- Update the Task 30 Phase 3 level-local scan primitive note.

## Validation

- `cargo test route_ -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this slice. This is a pure helper factoring step;
multi-level scan coordination and relation-backed recursive hierarchy loading
remain open.
