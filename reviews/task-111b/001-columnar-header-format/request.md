# Task 111b Review Request: Columnar Frozen-List Header Format

- Code commit: `05a3ec9e560540a80a684c83b00186437edfdc54`
- Packet: `reviews/task-111b/001-columnar-header-format`
- Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`

## Summary

This is the first Task 111b format checkpoint. It reserves durable tuple tag `0x29` for a versioned IVF columnar frozen-list header and adds encode/decode validation plus local `DataPage` / `DataPageChain` insertion and read helpers.

The header records list id, logical posting count, payload width, total heap TID count, deterministic column offsets, total column byte span, and first/last physical column block refs. Continuation/payload column pages are not implemented in this checkpoint.

## Implementation Notes

- New header layout is tagged `0x29`, versioned as `1`, and has zeroed reserved flags.
- Offset validation is derived from the logical shape:
  - gammas
  - payload bytes
  - heap TID counts
  - heap TID offsets
  - heap TIDs
  - rerank TIDs
  - deleted bitmap
- Decode rejects wrong tag/version, nonzero reserved flags, invalid or inverted column block ranges, zero counts/widths, inconsistent heap TID totals, and offset drift.
- Existing row/dense/aligned/packed tuple tags are unchanged.
- No scan-path behavior is switched to the new format in this slice.

## Validation

See `artifacts/manifest.md`.

- `cargo test -q columnar_frozen_list_header --lib`
  - `2 passed; 0 failed; 0 ignored; 0 measured; 2117 filtered out`
- `cargo test -q ivf_tuple_roundtrips --lib`
  - `2 passed; 0 failed; 0 ignored; 0 measured; 2117 filtered out`
- `cargo test -q layout_fit_helpers_track_page_capacity --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2118 filtered out`
- `rustfmt --check src/am/ec_ivf/page.rs`
  - passed for the touched file
