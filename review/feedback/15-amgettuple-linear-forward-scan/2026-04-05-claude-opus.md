# Feedback: `amgettuple` Linear Forward Scan Bootstrap

Request:
- `review/15-amgettuple-linear-forward-scan.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is the cursor advancement logic safe?

**Yes, with one edge case worth documenting.** The cursor logic in `next_linear_scan_heap_tid` (lines 540-622) is correct. Walking through the specific scenarios:

1. **Page boundaries:** When a page is exhausted (`offset > line_pointer_count`), the outer `for` loop advances `block_number`, `next_block_number` is updated at line 621, and `next_offset_number` resets to 1. Correct.

2. **Exhausted scan:** After all blocks are scanned, `scan_exhausted = true` at line 625. Subsequent calls return `None` at line 549. Correct.

3. **Rescan after partial drain:** `reset_scan_position` (line 505) resets `next_block_number` to `FIRST_DATA_BLOCK_NUMBER`, `next_offset_number` to 1, `scan_exhausted` to false, and clears pending heap TIDs. This means rescan starts from the beginning regardless of where the previous scan stopped. Correct.

4. **Rescan after exhaustion:** Same as above — the exhausted flag is cleared by `reset_scan_position`. The `debug_gettuple_rescan_after_exhaustion` test at line 2139 explicitly covers this. Correct.

5. **Skipped tuples:** Neighbor tuples are skipped at line 597 (tag check). Deleted or empty-heaptid elements are skipped at line 604. The cursor advances past them. Correct.

**Edge case: `next_offset_number` overflow at line 609.** `offset.saturating_add(1)` where `offset` is `u16`. If `offset == u16::MAX` (65535), `saturating_add(1)` produces 65535 (not 0). But line 610 checks `if opaque.next_offset_number == 0`, which handles the *wrapping* case, not the *saturation* case. Since `saturating_add` can't produce 0, this branch is dead code. The real concern is: could `offset` ever be `u16::MAX`? No — `page_line_pointer_count` returns the count of item pointers, and an 8KB page can hold at most ~1000 items. So `offset` is always far below `u16::MAX` and `saturating_add(1)` always produces `offset + 1`. **Safe, but the overflow guard at line 610 is dead code.** Consider removing it or replacing with a `debug_assert!(opaque.next_offset_number > offset)`.

### Is storing pending duplicate heap TIDs in scan opaque state the right boundary?

**Yes.** The `pending_heaptids` array in `TqScanOpaque` (line 1303) is a fixed-size inline array of `HEAPTID_INLINE_CAPACITY` (10) entries, matching the maximum inline heap TID count in `TqElementTuple`. This avoids allocation and keeps the drain state in a single contiguous struct.

**No stale-state risk across rescans:** `reset_scan_position` zeros `pending_heaptid_count` and `pending_heaptid_index`, which logically clears the pending list. The array contents are not zeroed, but `take_pending_scan_heap_tid` checks the count/index bounds first, so stale array values are never read. The `debug_gettuple_rescan_after_partial` test at line 2231 explicitly verifies this.

**No stale-state risk across amendscan:** `amendscan` frees the entire opaque struct via `pfree`, so all cursor state is destroyed. A subsequent `ambeginscan` would allocate fresh state. Correct.

### Is there a missing regression test?

**Two gaps identified:**

1. **Duplicate-heavy scans spanning page boundaries.** The current duplicate-rescan test (line 2231) uses only 3 rows, which all fit on a single data page. Consider adding a test with enough duplicate-coalesced rows that they span multiple pages, to verify the cursor correctly advances across page boundaries while draining pending heap TIDs from the previous page.

2. **Scan with mixed element types per page.** The build process interleaves element and neighbor tuples on the same page. The linear scan skips neighbor tuples via the tag check. A test that explicitly verifies the scan returns the correct count of heap TIDs for an index built with `m > 0` (where neighbor tuples are present) would increase confidence. The existing tests do use built indexes with neighbors, so this is likely covered implicitly, but an explicit assertion on the count would be stronger.

## Additional Findings

### Buffer locking pattern is correct but chatty

Each `amgettuple` call that advances to a new element tuple acquires a SHARE lock on the current page, reads one element tuple, then releases. For a sequential forward scan, this means acquiring and releasing the lock once per element tuple, even though many elements may be on the same page.

A more efficient pattern would be to hold the page lock while draining all elements from the current page, only releasing when advancing to the next page. This would reduce lock acquisition overhead from O(tuples) to O(pages). Not a correctness issue — flagging for the optimization phase.

### `RelationGetNumberOfBlocksInFork` called per gettuple

At line 553, `block_count` is fetched on every `amgettuple` call that doesn't have pending heap TIDs. For a non-empty index, this means one `RelationGetNumberOfBlocksInFork` call per element tuple. This is cheap (cached in the relcache) but could be cached in the opaque state during `amrescan`. Low priority.
