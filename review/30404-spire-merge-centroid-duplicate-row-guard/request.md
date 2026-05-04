# Review Request: SPIRE Merge Centroid Duplicate Row Guard

## Summary

Task 30 SPIRE Phase 2 now rejects duplicate affected leaf snapshot rows while
building scheduled merge replacement centroids.

Changes:
- Replace map collection with explicit affected-row insertion.
- Reject duplicate snapshot rows for affected merge leaf PIDs.
- Extend focused rejection coverage and update the Phase 2 checklist.

## Validation

- `cargo test scheduled_merge_replacement_centroid --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This tightens the helper added in `review/30403-spire-scheduled-merge-centroid-helper`.
No measurement claims; no PG callback coverage.
