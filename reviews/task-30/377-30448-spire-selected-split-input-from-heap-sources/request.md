# Review Request: SPIRE Selected Split Input From Heap Sources

## Summary

Task 30 SPIRE Phase 2 now has the relation wrapper that builds selected split
execution input directly from heap source rows.

Changes:
- Add
  `build_relation_selected_scheduled_split_replacement_execution_input_from_heap_sources`.
- Collect folded selected split rows from the active snapshot.
- Fetch indexed source vectors for those rows using the existing split source
  fetch helper.
- Delegate to the checked selected-plan source execution-input builder.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims. This compiles the PG-facing wrapper but does not yet add
a scheduler SQL/callback entrypoint or publish a replacement epoch.
