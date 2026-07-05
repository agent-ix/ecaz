# Review Request: SPIRE Scheduled Merge Centroid Helper

## Summary

Task 30 SPIRE Phase 2 now has a pure helper for recomputing the single
replacement centroid for scheduled merge execution.

Changes:
- Build the merge centroid from affected parent-routing child centroids.
- Weight non-empty leaves by active snapshot effective assignment count.
- Use equal weighting when all affected merge leaves are empty.
- Reject missing/stale leaf snapshot rows, parent PID drift, missing parent
  children, bad dimensions, and non-finite child centroids.
- Update the Phase 2 checklist.

## Validation

- `cargo test scheduled_merge_replacement_centroid --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This closes the merge-side centroid recomputation slice. Split centroid
training/routing remains open because it needs source-vector based routing.
No measurement claims; no PG callback coverage.
