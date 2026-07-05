# Review Request: SPIRE Split Source-Row Hydration

## Summary

Task 30 SPIRE Phase 2 now has a pure source-row hydration helper for split
replacement execution.

Changes:
- Add `SpireSplitReplacementFetchedSourceVector`.
- Add `build_split_replacement_source_rows`, which combines the selected split
  leaf's folded assignment rows with fetched source vectors keyed by heap TID.
- Preserve assignment row order while requiring exact source coverage and
  rejecting stale row groups, duplicate source TIDs, duplicate assignment TIDs,
  missing vectors, and unused vectors.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This keeps the next live heap
fetching slice focused on producing exact `(heap_tid, source_vector)` pairs for
the selected split leaf.
