# Review Request: SPIRE Recursive Leaf Count

## Summary

Task 30 Phase 3 scan planning now has a pure helper for counting actual
leaf-level children in a recursive routing hierarchy.

Changes:

- Add `count_snapshot_recursive_leaf_pids(...)`, which loads the active
  root/internal hierarchy and counts level-1 routing children.
- Add `count_recursive_routing_leaf_pids(...)` to traverse root/internal
  routing objects without a query vector.
- Factor recursive internal-child validation into
  `require_recursive_internal_child(...)` and reuse it from both route descent
  and leaf-count traversal.
- Preserve `count_snapshot_single_level_leaf_pids(...)` as the current
  root-child compatibility helper.
- Add local object-store coverage showing a recursive hierarchy with two root
  children but three leaf-level children.
- Update the Task 30 Phase 3 scan primitive note.

## Validation

- `cargo test count_snapshot_ -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this pure scan-helper slice. Per-level `nprobe`
resolution and relation-backed recursive SQL smoke remain open.
