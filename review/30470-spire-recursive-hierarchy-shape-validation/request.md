# Review Request: SPIRE Recursive Hierarchy Shape Validation

## Summary

Task 30 Phase 3 hierarchy diagnostics now validate recursive-capable active
metadata before level-aware scan code exists.

Changes:

- Add a pure `validate_recursive_hierarchy_shape` helper over active object
  summaries.
- Accept the existing single-level shape: one root at level 1 with level-0 leaf
  children.
- Accept the first recursive shape: a root above internal routing objects, with
  internal routing objects above level-0 leaves.
- Reject malformed active shapes such as root-to-leaf level skips,
  parent/child PID drift, duplicate active PIDs, duplicate children, missing
  child objects, nonzero root parent PID, zero-level routing objects, nonzero
  leaf/delta levels, and deltas whose parent is not an active leaf.
- Wire the validator into `ec_spire_index_hierarchy_snapshot(index_oid)` so an
  active malformed hierarchy reports `status = 'invalid_hierarchy_shape'`
  instead of looking recursively usable.
- Preserve the existing single-level diagnostic status and the explicit
  `recursive_routing_supported = false` /
  `per_level_nprobe_supported = false` flags while recursive build and scan
  are still open.
- Update the Task 30 Phase 3 hierarchy metadata note.

## Validation

- `cargo test recursive_hierarchy_shape -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this slice; the change is a pure metadata-shape
validator plus diagnostic wiring, and the existing SQL hierarchy snapshot
coverage exercises the preserved single-level status.
