# Task 111b Review Request: Columnar Buffer Chunks

- Code commit: `c29cf4e1df1dc9323a98ef2572408cebac0bb9b6`
- Packet: `reviews/task-111b/002-columnar-buffer-chunks`
- Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`

## Summary

This checkpoint adds the deterministic in-memory column buffer primitive that the 111b writer/reader will use before the physical PostgreSQL column-page write path is switched on.

It converts a frozen list of single-heap-TID build postings into parallel LE byte columns:

- `gamma[]`
- `payload[]`
- `heap_tid_count[]`
- `heap_tid_offset[]`
- `heap_tid[]`
- `rerank_tid[]`
- deleted bitmap

It also adds a page-aware payload chunk iterator that splits payload bytes by raw page capacity while preserving whole-posting boundaries. That matches the 111b durable-layout requirement that payload runs never split a posting across pages.

## Implementation Notes

- `IvfColumnarFrozenListColumns::from_single_heaptid_postings` validates non-empty input, nonzero payload width, finite gammas, and fixed payload width.
- The column buffer can derive the `0x29` header shape from packet 001.
- The payload chunker returns `(start_item, item_count, bytes)` chunks and rejects item widths too large for the page.
- This still does not switch build or scan behavior. It is the preparatory page-aware data model for the next writer/reader slice.

## Validation

See `artifacts/manifest.md`.

- `cargo test -q columnar_frozen_list --lib`
  - `5 passed; 0 failed; 0 ignored; 0 measured; 2117 filtered out`
- `rustfmt --check src/am/ec_ivf/page.rs`
  - passed for the touched file
