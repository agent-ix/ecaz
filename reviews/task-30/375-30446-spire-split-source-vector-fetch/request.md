# Review Request: SPIRE Split Source-Vector Fetch

## Summary

Task 30 SPIRE Phase 2 now has a relation-ready source-vector fetch bridge for
split replacement execution.

Changes:
- Expose the existing SPIRE heap-rerank indexed-vector heap loader as
  `load_indexed_source_vector_from_heap_row`.
- Reuse that loader from update mechanics via
  `fetch_split_replacement_source_vectors`.
- Return fetched `(heap_tid, source_vector)` records for folded replacement
  rows, ready for the source-row hydration and split materialization helpers.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims. This compiles the PG-facing fetch bridge but does not
yet wire a new callback or SQL surface; selected-plan invocation remains the
next live scheduler slice.
