# Feedback: Shared AM Helper Boundary

Request:
- `review/75-shared-am-helper-boundary.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is `shared` the right home for remaining common AM helpers?

**Yes.** Verified: `shared.rs` (344 lines) contains metadata page I/O (`read_metadata_page`, `initialize_metadata_page`), page-level utilities (`page_item_id`, `page_line_pointer_count`), heap TID decoding (`decode_heap_tid`), and debug/test helpers for index inspection. These are used by `build.rs`, `insert.rs`, `scan.rs`, and `vacuum.rs` — genuinely shared across behavior-specific modules.

`scan.rs` references `super::shared::page_line_pointer_count` (line 863) and `super::shared::page_item_id` (line 871) during linear page iteration. `insert.rs` uses `shared::decode_heap_tid` and `shared::read_metadata_page`. The helpers are correctly scoped.

### Does any helper belong in a behavior-specific module instead?

No. Every function in `shared.rs` is used by at least two behavior-specific modules. The debug helpers (`debug_index_metadata`, `debug_index_pages`, `debug_vacuum_stats`) are test infrastructure used across pg regression tests — they belong in the shared surface.

### Is `mod.rs` thin enough to stay stable?

**Yes.** At 50 lines it's purely structural: module declarations, constants, and cfg-gated re-exports. No implementation logic. Future traversal work will land in `scan.rs`, `search.rs`, or new modules — not in `mod.rs`.

## Additional Findings

No issues found. The four-way split (mod.rs root → build/insert/scan/search + shared) is clean.
