# Review Request: Duplicate Coalescing And Inline TID Capacity

Scope:
- `src/am/mod.rs`
- `src/am/page.rs`
- `src/lib.rs`

What changed:
- `aminsert` now coalesces duplicate encoded vectors into an existing element tuple rather than always appending a new element.
- Coalescing is limited by the current inline heap-TID capacity of the tuple format.
- When the tuple is already at capacity, insert rejects the duplicate instead of silently corrupting or overflowing state.

Review focus:
- Duplicate detection semantics
- Heap-TID append safety and bounds enforcement
- Interaction between duplicate coalescing and page reuse
- Whether failure mode/messages are coherent enough for current capability

Questions to answer:
- Is duplicate matching too weak or too strong for the stored encoding?
- Is the capacity boundary enforced exactly once and in the right place?
- Is there any path that can partially mutate a tuple before rejecting overflow?

---

## Review Comments

### 1. Duplicate detection does a full sequential scan of all data pages (performance, not correctness)

`find_duplicate_element_tid` (line 704-771) scans every data block from `FIRST_DATA_BLOCK_NUMBER` to `block_count`, reading each element tuple and comparing the full code byte-for-byte. For the current narrow scope this is acceptable, but it's O(n) per insert. The `dimensions` and `bits` parameters are passed in but unused (lines 768-769 `let _ = dimensions; let _ = bits;`), suggesting a future optimization path. No bug here.

### 2. Duplicate matching compares the full encoded `code` — this is the right granularity

The match at line 756 (`element.code == code`) compares the entire quantized code vector. Since two different input vectors can map to the same quantized code (that's the nature of product quantization), this correctly coalesces at the encoding boundary. The coalesced heap-TIDs then point to all heap rows that share the same encoded representation. This matches the semantic intent.

### 3. Capacity check is correctly placed — no partial mutation before rejection

In `coalesce_duplicate_heap_tid` (line 773-845):
1. The page is locked exclusive (line 795)
2. The existing element tuple is decoded (line 815-816)
3. If heap_tid is already present, the function returns early with no mutation (line 817-821) — correct idempotency
4. The capacity check happens at line 822 *before* the push at line 828
5. If at capacity, `pgrx::error!` is called, which longjmps out — the WAL transaction in scope will be aborted by Drop

This is **safe**: the error fires before any mutation to the decoded tuple, and the WAL txn is never finished, so no partial write reaches disk. The GenericXLog abort path handles this correctly.

### 4. The in-place update writes to the WAL-registered copy correctly

At line 840, `ptr::copy_nonoverlapping` writes the re-encoded tuple directly into the WAL-registered page image (the pointer came from `wal_txn.register_buffer` at line 797-798). The length equality check at line 832-837 ensures the fixed-size tuple format is preserved. This is correct — element tuples always have the same encoded length regardless of how many heap-TIDs are populated (the unused slots are filled with `INVALID`).

### 5. `find_duplicate_element_tid` releases the shared lock before `coalesce_duplicate_heap_tid` takes an exclusive lock

At line 757, when a duplicate is found, the buffer is unlocked (`UnlockReleaseBuffer`) before returning the ItemPointer. Then `coalesce_duplicate_heap_tid` re-opens the buffer with an exclusive lock. Between these two operations, another concurrent backend could:
- Delete the element tuple (though vacuum is a no-op currently, so this can't happen yet)
- Coalesce a different heap-TID into the same element (race to the capacity limit)

The second case means two concurrent inserters of different heap rows with the same code could both see `heaptids.len() < HEAPTID_INLINE_CAPACITY`, both proceed to coalesce, and the second one would find the TID count one higher than expected. The capacity check at line 822 still fires correctly in this case — worst case, one insert errors out when it could have succeeded with better serialization. This is **safe** but could produce a spurious rejection under high concurrency at exactly the capacity boundary.

For the current narrow path this is fine. If it matters later, holding the buffer lock across find + coalesce would close the gap.

### 6. Test coverage is comprehensive

- `test_tqhnsw_insert_coalesces_duplicate_vectors` (lib.rs:1288) verifies tuple count doesn't grow and heaptid count does
- `test_tqhnsw_insert_rejects_duplicate_heaptid_overflow` (lib.rs:1359) fills all 10 slots then confirms the 11th errors
- Both tests use `encode_to_tqvector` with identical inputs to guarantee code equality

**No missing tests** for the current scope. The idempotency path (inserting a row whose heap-TID is already coalesced) isn't explicitly tested, but it's hard to trigger via SQL since PostgreSQL assigns unique heap TIDs.
