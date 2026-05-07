# Review Request: SPIRE Merge Centroid Shared Duplicate Validation

## Summary

Task 30 SPIRE Phase 2 now has the merge centroid helper relying on the shared
scheduler row validator for duplicate leaf rows.

Changes:
- Remove the now-redundant affected-row duplicate check from the centroid
  helper.
- Keep duplicate coverage in the merge-centroid filter while expecting the
  shared scheduler validation error.

## Validation

- `cargo test scheduled_merge_replacement_centroid --lib`
- `cargo test replacement_scheduler --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This aligns the helper introduced in 30403/30404 with the broader selector row
validation introduced in 30406.
No measurement claims; no PG callback coverage.
