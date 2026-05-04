# Review Request: SPIRE Split Materialization From Rows

## Summary

Task 30 SPIRE Phase 2 now has a pure composition helper for split replacement
materialization.

Changes:
- Add `build_split_replacement_leaf_materialization_from_rows`.
- Compose source-row hydration with split centroid training/routing, so callers
  can pass folded selected-leaf rows plus fetched heap source vectors.
- Add focused coverage that fetched vectors may arrive out of assignment order
  while materialization still routes rows under the planned replacement PIDs.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This leaves the remaining live
split scheduler work centered on fetching source vectors and invoking the
selected-plan path under the publish lock.
