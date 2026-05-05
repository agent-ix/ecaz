# Review Request: SPIRE Recursive Centroid Records

## Summary

Task 30 Phase 3 recursive routing drafts now emit materialized centroid records
for every routing parent/child edge.

Changes:

- Add `SpireRecursiveCentroidRecord` with parent PID, child PID, child level,
  centroid ordinal, dimensions, centroid vector, and source count.
- Extend `SpireRecursiveRoutingBuildDraft` to carry centroid records alongside
  materialized root/internal routing objects.
- Emit records for both the single-level root-to-leaf shape and recursive
  root/internal-to-child shapes.
- Validate centroid records for finite dimensions, nonzero source count, and
  duplicate parent/child records.
- Extend focused recursive routing build tests to assert centroid-record output
  for both flat and internal-level drafts.
- Update the Task 30 Phase 3 centroid materialization note.

## Validation

- `cargo test recursive_routing_build -- --nocapture`
- `git diff --check`

## Notes

No relation I/O or PG18 SQL test was run for this slice. Durable storage and
SQL diagnostics for recursive centroid records remain open.
