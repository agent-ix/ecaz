# Review Request: SPIRE Recursive Routed Leaf Rows

## Summary

Task 30 Phase 3 scan row collection now uses the recursive-capable routing
preload path.

Changes:

- Add `SpireRecursiveLeafRoute`, carrying the selected leaf PID and its
  immediate routing-parent PID.
- Factor the recursive route coordinator through a route-returning helper while
  keeping the existing PID-only helper for focused route tests.
- Update `collect_snapshot_routed_probe_leaf_rows(...)` to load the active
  root/internal hierarchy, route recursively, and collect leaf rows using the
  selected leaf's immediate parent PID.
- Preserve the hierarchy root PID in `SpireRoutedLeafScanRows` for current scan
  result context.
- Keep the single-level path compatible because level-1 roots produce routes
  whose parent PID is the root PID.
- Add local object-store coverage for a root -> internal -> leaf hierarchy where
  recursive routed row collection would previously fail the root-only parent
  check.
- Update the Task 30 Phase 3 scan primitive note.

## Validation

- `cargo test collect_snapshot_routed_probe_leaf_rows_accepts_recursive_leaf_parent -- --nocapture`
- `cargo test collect_snapshot_routed_probe_leaf_rows -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this pure scan-helper slice. Quantized candidate
collection and relation-backed recursive SQL smoke remain open.
