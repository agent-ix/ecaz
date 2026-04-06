# Feedback: Scan Visited Seed State

Request:
- `review/53-scan-visited-seed-state.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Note: Line Numbers Updated

Function locations have shifted due to module extraction and beam scheduler additions. Visited-state semantics unchanged.

## Answers to Review Questions

### Is a heap-allocated `HashSet<ItemPointer>` the right first visited-state representation?

**Yes.** `HashSet<ItemPointer>` provides O(1) lookup and insertion, which is what traversal needs. The heap allocation via `Box::into_raw`/`Box::from_raw` follows the same pattern as the candidate frontier and prepared query — raw pointer in the `repr(C)` opaque struct, with explicit alloc/free lifecycle. This is consistent with the existing scan state ownership model.

For the expected traversal scale (ef_search up to 1000, graph fan-out ~M=16), the visited set could grow to a few thousand entries. A `HashSet` handles this efficiently. A bitset would be more compact if element TIDs were dense, but they're not — TIDs are (block, offset) pairs with sparse block allocation. `HashSet` is the right choice.

### Is seeding from valid frontier candidates too eager?

**No — it's the right precursor.** The visited set is seeded from the entry candidate and its successor (scan.rs:493, then via `refill_candidate_frontier_from_source` at scan.rs:670). These are the elements that have already been scored and placed in the frontier. Marking them as visited prevents re-scoring them when the frontier expands later. This is exactly the invariant that real traversal needs: "everything in the frontier or already consumed is visited."

Seeding too late (e.g., only on first consumption) would create a window where expansion could re-discover the entry or successor as a neighbor of some other node, leading to duplicate frontier entries.

### Missing lifecycle edges?

No gaps found. The lifecycle is:
- `reset_scan_visited_state` (scan.rs:382-388): called from `reset_scan_position` on rescan — allocates or clears
- `free_scan_visited_set` (scan.rs:390-395): called from `amendscan` — frees
- `mark_visited_element` (scan.rs:397-403): guards against null pointer and INVALID TID
- `visited_contains_element` (scan.rs:405-411): same guards

The null-pointer and INVALID-TID guards in both `mark_visited_element` and `visited_contains_element` are appropriate defensive code. The set is cleared on rescan (via `reset_scan_position` → `reset_scan_visited_state`), which is correct — a new scan starts with a fresh visited set.

## Additional Findings

No issues found.
