# Review Request: SPIRE Recursive Epoch Leaf Parent Validation

## Summary

Task 30 Phase 3 local recursive epoch materialization now validates that leaf
placements point to leaf objects whose stored parent matches the level-1
routing parent.

Changes:

- Derive expected leaf -> parent PID mappings from the recursive routing draft.
- Validate each caller-provided leaf placement by reading its object header.
- Reject non-leaf object placements.
- Reject leaf parent drift before building the manifest/directory snapshot.
- Preserve missing/extra leaf placement coverage validation.
- Add focused parent-drift rejection coverage.
- Update the Task 30 Phase 3 recursive build coordinator note.

## Validation

- `cargo test local_recursive_routing_epoch_draft -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this local build-helper hardening slice.
Relation-backed recursive build/publish wiring remains open.
