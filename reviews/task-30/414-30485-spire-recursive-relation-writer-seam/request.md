# Review Request: SPIRE Recursive Relation Writer Seam

## Summary

Task 30 Phase 3 recursive epoch materialization now has a relation-backed
object-store seam while preserving the existing local helper behavior.

Changes:

- Add `SpireRecursiveRoutingEpochObjectStore`, extending `SpireObjectReader`
  with recursive routing-object writes.
- Implement the seam for `SpireLocalObjectStore`.
- Implement the seam for `SpireRelationObjectStore`.
- Refactor `build_local_recursive_routing_epoch_draft(...)` through the shared
  writer path.
- Add `build_relation_recursive_routing_epoch_draft(...)` as the relation
  entry point for future recursive build/publish orchestration.
- Generalize leaf-placement header validation to the `SpireObjectReader`
  boundary.
- Update the Task 30 Phase 3 recursive build coordinator note.

## Validation

- `cargo test local_recursive_routing_epoch_draft -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this writer-seam slice. The relation entry point
is not yet called by live recursive build orchestration.
