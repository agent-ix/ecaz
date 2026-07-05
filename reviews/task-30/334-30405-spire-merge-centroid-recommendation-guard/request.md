# Review Request: SPIRE Merge Centroid Recommendation Guard

## Summary

Task 30 SPIRE Phase 2 now requires every affected merge row to still be
merge-recommended before computing the scheduled merge replacement centroid.

Changes:
- Reject affected rows that are no longer marked `merge_recommended`.
- Extend the focused stale-input coverage.
- Update the Phase 2 checklist.

## Validation

- `cargo test scheduled_merge_replacement_centroid --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This tightens the scheduled merge centroid helper so stale advisory rows fail
closed at the centroid-building seam too.
No measurement claims; no PG callback coverage.
