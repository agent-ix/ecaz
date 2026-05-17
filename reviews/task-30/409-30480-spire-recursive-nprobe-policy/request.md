# Review Request: SPIRE Recursive Nprobe Policy

## Summary

Task 30 Phase 3 recursive routing now follows the design-note default for
per-level `nprobe`.

Changes:

- Add `SpireRecursiveNprobePolicy`, currently carrying the configured
  leaf-level `nprobe`.
- Apply the configured relation/session `nprobe` at level 1.
- Apply the conservative default of one child at routing levels above level 1
  until durable per-level controls land.
- Keep the public recursive routing helper signature unchanged.
- Add focused coverage showing `nprobe = 2` probes two leaf-level children
  under the selected internal parent, but does not probe two upper-level
  internal children.
- Update the Task 30 Phase 3 scan primitive note.

## Validation

- `cargo test route_recursive_routing_objects_to_leaf_pids -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this pure routing-policy slice. Durable per-level
metadata, reloption exposure, and recursive SQL smoke remain open.
