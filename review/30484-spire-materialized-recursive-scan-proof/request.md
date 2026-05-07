# Review Request: SPIRE Materialized Recursive Scan Proof

## Summary

Task 30 Phase 3 now has a local proof that a materialized recursive routing
epoch can be scanned through the recursive quantized candidate path.

Changes:

- Build a recursive routing hierarchy draft from four leaf centroid inputs.
- Write matching V2 leaf objects into the local object store using the level-1
  routing parent PIDs from the draft.
- Materialize a local recursive routing epoch through
  `build_local_recursive_routing_epoch_draft(...)`.
- Build a published snapshot from that materialized epoch.
- Scan the snapshot with `collect_quantized_routed_probe_candidates(...)`.
- Assert the expected top two positive-side candidates are returned in order.
- Update the Task 30 Phase 3 review-packet status note.

## Validation

- `cargo test materialized_recursive_routing_epoch_scans_quantized_candidates -- --nocapture`
- `git diff --check`

## Notes

No PG18 SQL test was run for this local proof slice. Relation-backed recursive
build/publish wiring remains open before SQL smoke is meaningful.
