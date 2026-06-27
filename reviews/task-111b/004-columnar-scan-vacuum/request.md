# Task 111b Review Request: Columnar Scan and Vacuum

- Code commit: `e631ac4c3ca8fa6ad71c46062aa93d952e0ed721`
- Packet: `reviews/task-111b/004-columnar-scan-vacuum`
- Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`

## Summary

This checkpoint makes the gated Task 111b columnar frozen-list format readable and vacuum-safe.

The scan visitor now recognizes the `0x29` columnar header, derives the expected raw-page byte lengths from the header and page size, copies only valid bytes out of each raw page special area, decodes a borrowed columnar list view, and drains live/nondeleted postings into the existing SOA scorer path. Row delta tuples in the same list range continue to scan normally.

Vacuum now runs a columnar-header pass that reads the logical column bytes, marks deleted postings in the columnar deleted bitmap when all heap TIDs for that posting are dead, rewrites the raw pages, and keeps list/metadata live/dead counts correct alongside normal row/dense rewrites.

## Implementation Notes

- Adds `IvfColumnarFrozenListRef` over copied logical column bytes.
- Adds header-derived raw-page length calculation shared by writer/read tests and the read path.
- Adds raw-page read and rewrite helpers for the page special area.
- Leaves an 8-byte guard before the special area on raw column pages. A PG18 test exposed that initializing a page with special space starting exactly at `pd_lower` can abort during live index build; the guard still leaves too little free space for normal tuple insertion.
- Adds `IvfPostingEntryRef::ColumnarHeader`.
- Charges columnar decoded postings through the existing dense posting counters for this correctness slice.
- Adds PG18 coverage for build scan, mixed inserted row scan, vacuum deletion from the columnar bitmap, directory counts, and post-vacuum scan.

## Validation

See `artifacts/manifest.md`.

- `cargo test -q columnar_frozen_list_raw_pages_keep_all_column_items_whole --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2124 filtered out`
- `cargo test -q build_state_can_stage_columnar_frozen_lists_when_gated --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2124 filtered out`
- `cargo test -q test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2124 filtered out`
