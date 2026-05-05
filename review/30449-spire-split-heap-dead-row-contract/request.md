# Review Request: SPIRE Split Heap Dead-Row Contract

## Summary

Task 30 SPIRE Phase 2 now handles the reviewer-raised split heap-dead-row
contract before live SQL wiring.

Changes:
- Add `filter_split_replacement_rows_to_fetched_sources`.
- Keep fetched heap TIDs as the live assignment set for split materialization.
- Drop assignment rows whose heap tuple no longer fetches under the heap
  snapshot before exact source coverage validation runs.
- Reject duplicate fetched heap TIDs before filtering.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

Addresses review feedback in
`review/30448-spire-selected-split-input-from-heap-sources/feedback.md`.
No measurement claims; no PG callback coverage.
