# Task 111h / 004 - Rerank Group Chain Page APIs Checkpoint

Code commit: `e519d29bee8c4876f38ad2a89e33f5000747cd58`

## Summary

This checkpoint corrects and extends the packed rerank group page codec before
the build/scan integration starts emitting it durably.

What changed:

- Added a distinct `next_group_tid` field to the `0x2B` rerank group header.
  The existing `next_segment_tid` now has one job only: payload continuation
  segments. Group headers have their own chain for fallback scans, vacuum, and
  inspection.
- Updated the fixed header width and encode/decode roundtrip tests to pin both
  pointers.
- Added typed `DataPage` / `DataPageChain` APIs for inserting, updating, and
  reading rerank group headers and inserting/reading payload continuation
  segments.
- Added relation-level read helpers for `0x2B` group headers and `0x2C` payload
  segments.
- Extended page and page-chain roundtrip tests to cover the new staged APIs and
  header update flow.

This remains a preparatory page/storage API checkpoint. The writer and scan
paths still use the legacy `0x2A` direct-TID sidecar until the next integration
slice switches them over and bumps the durable format version.

## Code Changes

- `src/am/ec_ivf/page.rs`: adds `next_group_tid`, typed page APIs for `0x2B` /
  `0x2C`, relation-level read helpers, and focused tests.

No benchmark matrix is included in this packet.

## Validation

Artifacts are under `reviews/task-111h/004-rerank-group-chain-page-apis/artifacts/`.

- `cargo test --no-default-features --features pg18 rerank_group --lib`
  passed: `3 passed; 0 failed`.
- `cargo test --no-default-features --features pg18 data_page_ivf_tuple_roundtrips --lib`
  passed: `1 passed; 0 failed`.
- `cargo test --no-default-features --features pg18 data_page_chain_ivf_tuple_roundtrips --lib`
  passed: `1 passed; 0 failed`.
- `cargo check --no-default-features --features pg18` passed.

## Review Focus

- Confirm `next_group_tid` is the right correction rather than overloading
  payload continuation pointers for group-chain traversal.
- Confirm the staged page APIs are sufficient for build to create payload
  segments, link headers, and then hand direct header TIDs to postings.
- Confirm this remains pre-durable-writer work, with the format version bump
  intentionally deferred until build/insert paths emit the new tags.
