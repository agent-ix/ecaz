---
id: ADR-013
title: "Persist gamma in element tuples to eliminate heap fetches during scan and insert"
status: ACCEPTED
impact: HIGH for FR-007, FR-009, FR-010, FR-016
date: 2026-04-05
---
# ADR-013: Persist gamma in element tuples to eliminate heap fetches during scan and insert

## Context

The current `TqElementTuple` page layout stores only `code_bytes` for each indexed element. The
scalar `gamma` term — required for correct asymmetric scoring via the prepared-query path — is not
persisted in the index. Instead, it is recovered at runtime by fetching the representative heap row
(the first stored heap TID), detoasting the `tqvector` datum, and unpacking `gamma` from the wire
format.

This heap-fetch-per-element pattern appears in two critical paths:

1. **Duplicate detection during `aminsert`** (`find_duplicate_element_tid` in `mod.rs`): For every
   candidate element with matching code bytes, the representative heap row is fetched to compare
   `gamma`. This is O(element_count) heap fetches in the worst case.

2. **Scan scoring** (`score_scan_element_result` in `scan.rs`): Every element scored during tuple
   production requires a heap fetch to recover `gamma`, then constructs a temporary
   `[gamma][code_bytes]` `Vec<u8>` payload to pass to the quantizer scorer.

For the current bootstrap linear scan, these costs are acceptable because results are unordered and
the scan visits each element at most once. For upcoming ordered traversal with `ef_search`
controlling beam width, each `amgettuple` call will explore potentially hundreds of candidates.
The heap fetch becomes the dominant cost — each one requires a buffer pin, page read, detoast,
unpack, and buffer release.

Additionally, the representative-heap-row dependency is fragile: if the first heap TID in a
coalesced element is deleted by vacuum or HOT pruning while other heap TIDs survive, the gamma
fetch fails. Future vacuum work would need to handle representative row migration, adding
complexity to what should be a simple index-local operation.

## Decision

Add a `gamma: f32` field (4 bytes, stored as little-endian) to `TqElementTuple` in the persisted
page layout.

Specifically:

- `TqElementTuple` encoding gains a 4-byte `gamma` field after the inline heap-TID count byte and
  before the stored neighbor TID pointer.
- `TqElementTuple` decoding reads `gamma` from the new position.
- `GraphElement` gains a `gamma: f32` field populated from the decoded element tuple.
- Build-time tuple construction persists `gamma` from the source `tqvector` datum.
- Live `aminsert` persists `gamma` from the inserted `tqvector` datum.
- Duplicate detection in `find_duplicate_element_tid` compares `element.gamma.to_bits()` directly
  against the incoming tuple's `gamma.to_bits()` without a heap fetch.
- Scan scoring reads `gamma` from the element tuple (via `GraphElement` or direct decode) instead
  of fetching the representative heap row.
- `heap_tqvector_gamma` remains available as a fallback for any path that genuinely needs to read
  the heap (e.g., future consistency checks), but is no longer on the scan or insert hot path.

The quantizer scorer should also gain a `score_ip_from_parts(prepared: &PreparedQuery, gamma: f32,
code_bytes: &[u8]) -> f32` method that avoids the intermediate `Vec<u8>` payload construction.

## Consequences

### Benefits

- Eliminates heap fetches during scan scoring: O(candidates_explored) I/O savings per
  `amgettuple` call in ordered traversal.
- Eliminates heap fetches during duplicate detection: O(element_count) I/O savings per
  `aminsert` with code-byte matches.
- Removes the representative-heap-row dependency for gamma recovery, simplifying future vacuum
  and delete-marking logic.
- Enables zero-allocation scan scoring via `score_ip_from_parts`.
- Makes the element tuple self-contained for query scoring — no external state needed beyond the
  prepared query.

### Tradeoffs

- 4 bytes per element tuple. At 1536-dim 4-bit quantization, the current element tuple is ~842
  bytes, so this is a 0.5% size increase. Negligible impact on page density.
- Page-layout break: existing indexes built before this change will not have the `gamma` field.
  Since the format is pre-v1 and there is no deployed user base, this is acceptable. A version
  field in the metadata page could be added for forward compatibility, but is not required at this
  stage.
- Build and insert paths must ensure `gamma` is populated consistently. The value comes from the
  `tqvector` datum's wire format, which already stores `gamma` as the first 4 bytes of the
  payload. No new computation is needed.

## Follow-Up

1. After persisting `gamma`, remove the heap-fetch scoring path from `score_scan_element_result`
   and the heap-fetch duplicate check from `find_duplicate_element_tid`.
2. Add `ProdQuantizer::score_ip_from_parts` to eliminate the per-candidate `Vec<u8>` allocation.
3. Update `initialize_scan_entry_candidate` and successor seeding to read `gamma` from the graph
   element instead of scoring through the heap path.
4. Consider whether a metadata-page version field is worthwhile for future page-layout migrations.
