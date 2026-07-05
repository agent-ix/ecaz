# Review Request: SPIRE Recursive Quantized Candidates

## Summary

Task 30 Phase 3 quantized scan candidate collection now uses recursive routing
when the active hierarchy has internal routing objects.

Changes:

- Update `collect_quantized_routed_probe_candidates(...)` to load the active
  recursive routing hierarchy instead of only the root object.
- Add a recursive validated quantized helper that routes through root/internal
  objects and emits selected leaf routes with immediate parent PIDs.
- Factor the shared quantized scoring loop over `SpireRecursiveLeafRoute` so
  flat and recursive paths use the same candidate collection behavior.
- Keep the existing single-level validated helper for scan placement
  diagnostics, mapping root children to root-parent routes.
- Update quantized leaf parent validation to compare against the expected
  immediate parent PID, not always the root PID.
- Add local object-store coverage for root -> internal -> V2 leaf quantized
  candidate collection.
- Update the Task 30 Phase 3 scan primitive note.

## Validation

- `cargo test collect_quantized_routed_probe_candidates -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this pure scan-helper slice. Relation-backed
recursive SQL smoke and planner/reloption activation remain open.
