# Review Request: Insert Metadata Lock Scope

Scope:
- `src/am/insert.rs` — `tqhnsw_aminsert`
- `src/am/shared.rs` — `with_locked_metadata_page`

## Problem

`tqhnsw_aminsert` (insert.rs:32-69) runs the entire insert operation inside
`with_locked_metadata_page`, which holds an EXCLUSIVE lock on block 0 for the full duration.

The callback contains:

1. Shape validation
2. `find_duplicate_element_tid` — a full sequential scan of ALL data pages (insert.rs:265-333)
3. Either `coalesce_duplicate_heap_tid` or `append_heap_tuple`
4. Possible entry point update

Step 2 is O(n) in the number of index pages. Every data page is read with BUFFER_LOCK_SHARE inside
the exclusive metadata lock. This means:

- All concurrent inserts are fully serialized through one lock
- Each insert holds that lock for O(pages) buffer reads
- Bulk loading N rows into an index with E existing elements is O(N * E) in I/O

## Impact

This is the dominant factor in **test init time** for any test that inserts rows into a table with a
tqhnsw index. Each INSERT triggers a full index scan under an exclusive lock. For a 1000-row test
fixture, the 1000th insert scans ~1000 elements.

It also means concurrent inserts from multiple connections are completely serialized with O(n)
hold time per insert.

## Suggested Fix

Narrow the exclusive metadata lock to only the writes that need it:

1. Read metadata with BUFFER_LOCK_SHARE (get dimensions, bits, seed, entry_point)
2. Run `find_duplicate_element_tid` outside any metadata lock (just SHARE locks on data pages)
3. Acquire EXCLUSIVE on metadata only for the final write:
   - If duplicate: coalesce (no metadata change needed — skip metadata lock entirely)
   - If new element: append tuple, then update entry_point if needed under exclusive lock

Race between step 2 and step 3 (another insert creates the same duplicate): re-check after
acquiring the exclusive lock, or accept the rare double-insert and let the next scan coalesce.

The duplicate scan itself could also benefit from an early-exit optimization or a bloom filter, but
narrowing the lock scope is the highest-value change.

Please review whether the lock narrowing is safe given the current concurrency model, and whether
the duplicate re-check race matters for correctness.
