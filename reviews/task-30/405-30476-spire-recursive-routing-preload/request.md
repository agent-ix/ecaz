# Review Request: SPIRE Recursive Routing Preload

## Summary

Task 30 Phase 3 scan routing now has a checked preload seam for recursive
routing metadata.

Changes:

- Add `SpireLoadedRoutingHierarchy`, carrying the active root routing object
  and active internal routing objects by PID.
- Add `load_snapshot_routing_hierarchy(...)` over
  `SpireValidatedEpochSnapshot` plus the existing `SpireObjectReader`
  boundary.
- Keep `load_snapshot_root_routing_object(...)` as the current single-level
  compatibility wrapper by delegating through the new hierarchy loader.
- Skip non-routing objects while collecting root/internal routing objects.
- Preserve existing strict/degraded placement skip behavior through
  `should_skip_placement(...)`.
- Reject multiple active roots.
- Add local object-store coverage for root/internal loading and multiple-root
  rejection.
- Update the Task 30 Phase 3 scan primitive note.

## Validation

- `cargo test load_snapshot_routing_hierarchy -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this slice. Live recursive scan integration still
needs to call this preload seam before the recursive route coordinator.
