# Review Request: SPIRE Local Recursive Routing Epoch

## Summary

Task 30 Phase 3 build scaffolding can now materialize a recursive routing
draft into a validated local epoch snapshot when leaf placements already exist.

Changes:

- Add `SpireRecursiveRoutingEpochInput` and
  `SpireRecursiveRoutingEpochDraft`.
- Add `build_local_recursive_routing_epoch_draft(...)` for
  `SpireLocalObjectStore`.
- Write the recursive draft's root/internal routing objects into the local
  store.
- Combine routing placements with caller-provided leaf placements.
- Build object manifest and placement directory from durable object TIDs.
- Validate leaf placement coverage against level-1 routing children.
- Return a snapshot-validated epoch draft and conservative `next_pid`.
- Add focused success and missing-leaf-placement rejection tests.
- Update the Task 30 Phase 3 recursive build coordinator note.

## Validation

- `cargo test local_recursive_routing_epoch_draft -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this local build-helper slice. Relation-backed
recursive build/publish wiring remains open.
