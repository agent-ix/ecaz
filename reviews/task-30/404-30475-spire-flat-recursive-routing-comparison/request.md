# Review Request: SPIRE Flat vs Recursive Routing Comparison

## Summary

Task 30 Phase 3 now has a small pure flat-vs-recursive routing comparison.

Changes:

- Add a synthetic four-leaf comparison test.
- Build one flat single-level root over the four leaf centroids.
- Build one two-level recursive hierarchy over the same leaf centroids:
  root -> two internal routing objects -> four leaf PIDs.
- Route the same query through both shapes and assert both select the same best
  leaf.
- Update the Task 30 Phase 3 review-packet status to record that pure helper
  comparison exists while relation-backed SQL/candidate-scoring comparison
  remains open.

## Validation

- `cargo test recursive_route_matches_flat_single_level_on_small_hierarchy -- --nocapture`
- `git diff --check`

## Notes

No measurement claims. No PG18 SQL test was run for this slice; this is a pure
helper-level comparison before relation-backed recursive build/scan wiring.
