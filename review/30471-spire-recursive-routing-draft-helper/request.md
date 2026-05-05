# Review Request: SPIRE Recursive Routing Draft Helper

## Summary

Task 30 Phase 3 now has a pure in-memory recursive routing hierarchy draft
helper before relation-backed recursive build wiring.

Changes:

- Add `SpireRoutingPartitionObject::root_at_level(...)`, preserving the
  existing `root(...)` constructor as the level-1 single-level shorthand.
- Add recursive routing child/build/draft structs for child PID/centroid input
  and materialized routing-object output.
- Add `build_recursive_routing_hierarchy_draft(...)`, which:
  - accepts same-level child PID/centroid records
  - preserves the current single-level root shape when the child count is under
    target fanout
  - repeatedly trains spherical k-means over child centroids when another
    routing level is needed
  - allocates internal/root PIDs from the existing `SpirePidAllocator`
  - materializes root/internal `SpireRoutingPartitionObject`s with correct
    parent PIDs and levels
  - advances the allocator only after the draft validates
- Add focused unit coverage for single-level preservation, internal-level
  materialization, and invalid mixed child levels.
- Update the Task 30 Phase 3 recursive build coordinator status.

## Validation

- `cargo test recursive_routing_build -- --nocapture`
- `git diff --check`

## Notes

No relation I/O or PG18 SQL test was run for this slice. Live relation-backed
recursive build remains open; this checkpoint only adds the pure draft helper
that later build/publish code can consume.
