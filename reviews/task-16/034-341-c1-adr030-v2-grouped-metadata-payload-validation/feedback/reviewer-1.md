## Feedback: ADR-030 v2 Grouped Metadata Payload Validation

Read `GraphStorageDescriptor::from_metadata` in `src/am/graph.rs` lines 24-88.

### What's right

- Validation is now comprehensive for grouped-v2: payload flags
  (`PAYLOAD_FLAG_GROUPED_SEARCH_CODE`, `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD`), search
  codec kind (`GroupedPq`), search bits (`== 4`), non-zero search shape, and rerank
  codec kind (`ScalarQuantized`). Six distinct reject conditions, each with a
  specific error message. An operator who sees one of these knows exactly which
  metadata field is wrong.
- Validation happens at `from_metadata`, so it is load-bearing for every call site
  that builds a `GraphStorageDescriptor` (scan open, insert gate, vacuum gate,
  read helpers). One place, caught by everything downstream.
- Error messages are readable: "unsupported grouped-v2 search codec: {kind}" rather
  than opaque codes.

### Concerns

1. **Two validation sites now.** Packet 334 added a scorer-time payload-shape check
   against metadata-derived shape. Packet 341 adds a metadata-time check for the
   same payload flag. These are different invariants (metadata coherence vs.
   metadata-vs-tuple coherence), so both should exist. But the second validation
   site means a drift between them is possible: a future metadata field added to
   one path but not the other. Worth a regression test that asserts: if
   `from_metadata` accepts a metadata, the scan-time payload view check cannot
   reject it for shape reasons.

2. **`rerank_code_len` still uses `metadata.bits`.** Line 84:
   `rerank_code_len: crate::code_len(metadata.dimensions as usize, metadata.bits)`.
   I flagged this on packet 312. The convention is now: for grouped-v2, `bits`
   means "rerank codec bits" and `search_bits` means "search codec bits (4)".
   That's fine as long as the rerank codec is `ScalarQuantized`. If a future
   rerank codec differs in bit width, this computation silently uses the wrong
   number. Low priority, but worth a `// bits is load-bearing for rerank codec
   width` note near line 84 so it's not accidentally rewired.

3. **Incomplete `search_bits` check.** The metadata insists on `search_bits == 4`.
   For `GroupedPq` that's correct. If the search codec ever grows a different
   bit width, this check will reject it. That's appropriate for the current design
   but means the `GroupedPq` search codec is currently hardcoded to 4-bit. Worth
   naming in a comment.

### Observation

The metadata contract is now strict. Combined with packet 337 (insert) and 338
(vacuum), a grouped-v2 index is difficult to mishandle. Nice closure on the
safety runway.
