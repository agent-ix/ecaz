# Review Request: SPIRE Flat/Recursive Candidate Comparison

## Summary

Task 30 Phase 3 now has a pure flat-vs-recursive candidate comparison over
real V2 leaf objects in the local object store.

Changes:

- Add a four-leaf flat single-level snapshot with one root routing object.
- Add a matching two-level recursive snapshot with one root, two internal
  routing objects, and the same four leaf PIDs.
- Encode identical TurboQuant assignment payloads in both snapshots.
- Query both snapshots with the recursive-capable quantized candidate
  collector.
- Assert both shapes return the same top candidate PID, TID, and full candidate
  record.
- Update the Task 30 Phase 3 review-packet status note.

## Validation

- `cargo test recursive_quantized_candidates_match_flat_single_level_on_small_hierarchy -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this local object-store proof slice. Relation-
backed recursive SQL smoke remains open because recursive build/publish wiring
is not yet available.
