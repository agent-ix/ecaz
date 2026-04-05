# Review Request: Tail-Page Reuse And Rollover

Scope:
- `src/am/mod.rs`
- `src/am/page.rs`
- `src/lib.rs`

What changed:
- Live inserts no longer always require a fresh page.
- The AM now reuses the current tail data page when it has room for the neighbor+element pair.
- If the tail page cannot fit the pair, insert allocates a new data page and appends there.

Review focus:
- Free-space and tuple-fit accounting
- Page-selection logic for append targets
- Whether page rollover could leave metadata or tuple linkage inconsistent
- Whether current tests actually prove the intended boundary

Questions to answer:
- Is there an off-by-one or packing/alignment bug in the fit check?
- Could rollover choose the wrong page or mis-handle the first tuple on a new page?
- Is there a better minimal test for page-boundary behavior?

---

## Review Comments

Status at `9444d4b`:
- Comments 1-4 and 6: not needed. These notes validate accepted current rollover behavior rather than identifying a correctness defect that needs code changes now.
- Comment 5 addressed by adding rollover-followed-by-reuse regression coverage.

### 1. Free-space check uses `required_bytes` but PostgreSQL's `PageGetFreeSpace` accounts for different overhead

At line 526-527, `required_bytes` is computed using `page::raw_tuple_storage_bytes`, which adds `TUPLE_HEADER_BYTES (4) + LINE_POINTER_BYTES (4)` plus alignment to 8. But `PageGetFreeSpace` (line 576) returns `pd_upper - pd_lower - sizeof(ItemIdData)` — it already accounts for one line pointer but not for tuple headers or alignment the same way the Rust-side model does.

The Rust-side `raw_tuple_storage_bytes` includes `LINE_POINTER_BYTES` (4 bytes) in its calculation, and `PageGetFreeSpace` effectively subtracts `sizeof(ItemIdData)` (also 4 bytes) from the available space. For two tuples (neighbor + element), `required_bytes` includes 2 × LINE_POINTER_BYTES, but `PageGetFreeSpace` only deducts 1. This means the fit check is **conservative** — it may reject pages that actually have room, causing an early rollover. This is safe but wastes space.

**Suggestion:** Either trust `PageGetFreeSpace` and compare against payload-only sizes (without line pointer overhead for the second tuple), or do the accounting entirely on the Rust side by reading `pd_lower`/`pd_upper` directly. The current approach errs on the side of caution, which is fine for correctness.

### 2. Rollover drops the WAL transaction correctly

At lines 578-582, when the tail page doesn't fit, the code drops the `wal_txn` (which triggers `GenericXLogAbort` via the Drop impl) and releases the buffer before calling `append_heap_tuple_to_new_page`. The WAL module's Drop implementation (wal.rs:58-65) correctly aborts if `finish()` wasn't called. This is clean — no partial WAL record is committed.

### 3. The `append_heap_tuple_to_new_page` function duplicates the tuple-writing logic

`append_heap_tuple_to_new_page` (line 633-702) is essentially a copy of the second half of `append_heap_tuple` with `P_NEW` hardcoded. The duplication is acceptable for now given the narrow scope, but both paths construct the `TqElementTuple` identically. If the element construction ever diverges between the two, that's a subtle bug source.

### 4. Tail-page selection logic is correct

At lines 542-546, `target_block = existing_blocks - 1` when there are data blocks. This correctly picks the last (tail) data page. The `existing_blocks > FIRST_DATA_BLOCK_NUMBER` guard ensures we don't try to reuse the metadata page.

### 5. Test coverage for the boundary is good but could be tighter

`test_tqhnsw_insert_reuses_tail_page_when_space_remains` (lib.rs:1146) uses 4-dimensional vectors which are tiny and easily fit. The assertion `after_block_count == before_block_count` confirms reuse.

`test_tqhnsw_insert_allocates_new_page_when_tail_is_full` (lib.rs:1199) dynamically finds a dimension that saturates one page, which is a good approach — it exercises the exact boundary.

**One gap:** Neither test verifies that after rollover, *subsequent* inserts reuse the *new* tail page. A test that does: build → saturate tail → insert (triggers rollover) → insert again (should reuse new tail) would confirm the rollover + reuse cycle works end-to-end.

### 6. No risk of page-0 corruption on rollover

The `P_NEW` path always calls `PageInit` (line 655), so a freshly allocated page starts clean. The block number comes from `BufferGetBlockNumber` (line 657), not from arithmetic, so it's always correct regardless of concurrent relation extension. This is sound.
