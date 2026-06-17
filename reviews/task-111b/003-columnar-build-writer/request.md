# Task 111b Review Request: Gated Columnar Build Writer

- Code commit: `e7fed64dfa97870d93e68676f276cd7bff5b9cba`
- Packet: `reviews/task-111b/003-columnar-build-writer`
- Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`

## Summary

This checkpoint adds the build-time writer for the Task 111b columnar frozen-list format behind an explicit reloption gate: `columnar_frozen_lists = 1`.

When enabled for supported IVF storage formats, each non-empty frozen list is staged as:

- one versioned `0x29` header tuple in the normal data-page tuple stream
- raw physical column pages containing payload bytes in the page special area
- a list directory range whose head is the header block and whose tail is the last raw column block

The gate defaults off, and the existing row/dense/packed dense paths remain selected unless the new reloption is set.

## Implementation Notes

- Adds the `columnar_frozen_lists` reloption and plumbs it through `EcIvfOptions`.
- Adds build staging for `IvfColumnarFrozenListColumns` with deterministic header block refs.
- Writes raw column pages separately from normal tuple pages during `flush_build_plan`.
- Reserves the full raw payload area as page special space so normal tuple insertion will not reuse column payload pages.
- Adds a separator page after each raw column page run so subsequent list/directory tuple staging does not append normal tuples to the last raw page placeholder.
- Responds to packet-002 reviewer feedback by generating raw pages from item-aligned chunks for every column, not from an arbitrary flat byte split:
  - gamma: 4-byte items
  - payload: fixed payload-width items
  - heap TID counts: 2-byte items
  - heap TID offsets: 4-byte items
  - heap/rerank TIDs: `ItemPointer`-width items
  - deleted bitmap: byte items

## Scope Boundary

This still does not switch scan behavior. A columnar-gated index will need the next slice to read the header, derive expected per-raw-page byte lengths from the header/page size, copy raw bytes into scratch, and feed the existing scorer while continuing to scan row delta tuples in the same list range.

## Validation

See `artifacts/manifest.md`.

- `cargo test -q columnar_frozen_list --lib`
  - `7 passed; 0 failed; 0 ignored; 0 measured; 2117 filtered out`
- `cargo test -q build_state_can_stage_columnar_frozen_lists_when_gated --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2123 filtered out`
- `cargo test -q build_state_can_stage_dense_posting_blocks_when_gated --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2123 filtered out`
- `cargo test -q build_state_can_stage_packed_dense_posting_segments_when_requested --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2123 filtered out`
