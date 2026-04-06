# Feedback: Derived Frontier Head State

Request:
- `review/87-derived-frontier-head-state.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-06

## Response to Review Focus

### Is deriving head state on demand the right intermediate contract?

**Yes.** `current_candidate_frontier_head_tid` (scan.rs:704-712) computes the head freshly on each call:
1. First consults `scheduler_best_frontier_node` (beam authority, with stale-node purge)
2. Falls back to `visible_frontier_ref(opaque).best_tid_by_score()` (Vec linear scan)

This eliminates the `candidate_frontier_head: Option<usize>` field that was in `TqScanOpaque`. Confirmed the field is gone from the struct (scan.rs:1106-1129) — no cached head index exists.

The derived approach is correct because:
- The head was always logically derived (it was recomputed after every mutation anyway)
- Caching added a Vec-index coupling that had to be maintained across consume, refill, and reseed operations
- Derivation from the beam is O(1) via `peek_best()`, so there's no performance cost vs. a cached index
- The Vec fallback is O(n), but only fires when the beam is empty, which should be rare

### Do any remaining debug/runtime paths assume head is cached mutable state?

**No.** I verified:
- `scan_debug.rs` uses `current_candidate_frontier_head_tid(opaque)` (lines 627, 658, 684, 766, 775, 782, 816, 822, 827, 861, 891) — all go through the derived function
- No code reads or writes a `candidate_frontier_head` field directly
- The `consume_candidate_frontier_head` function (scan.rs:721-730) uses `scheduler_best_frontier_node` first, then falls back to `current_candidate_frontier_head_tid` for the Vec path
- No `recompute_candidate_frontier_head` function exists — it was the old mutation-then-cache function, now removed

The transition is complete.

### Should the next slice target the Vec-index mapping itself?

**Yes.** The remaining Vec-specific operation is `take_candidate_frontier_node` (scan.rs:714-718) which calls `visible_frontier_mut(opaque).remove_node(element_tid)`. This does a linear search by TID then `Vec::remove` — it's the last path that needs to translate a TID into a Vec position. Reviews 88-90 appear to address this by making consumption fully TID-driven and then removing the shared index-lookup helper.

## Additional Findings

No issues found. Clean removal of unnecessary mutable state.
