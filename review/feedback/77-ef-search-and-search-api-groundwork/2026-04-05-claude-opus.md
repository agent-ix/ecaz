# Feedback: ef_search And Search API Groundwork

Request:
- `review/77-ef-search-and-search-api-groundwork.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is the incremental `search.rs` API minimal but sufficient?

**Yes.** Verified the API surface in `search.rs`:
- `seed` / `seed_many` — initialize frontier candidates (used by `seed_discovered_candidates` at scan.rs:561 and `seed_existing_frontier_into_expansion` at scan.rs:568)
- `peek_best` — read best queued candidate without consuming (used in `recompute_candidate_frontier_head` at scan.rs:619 and `consume_candidate_frontier_head` at scan.rs:650)
- `expand_one` — pop best + add neighbors (used in `top_up_bootstrap_frontier` at scan.rs:602, currently with an empty neighbor iterator since the scan does its own neighbor loading)
- `forget_queued` — remove a consumed node from the scheduler (used in `consume_candidate_frontier_head` at scan.rs:656)
- `frontier_snapshot` / `frontier_len` / `visited_count` / `discovered` — observability for debugging

This covers the current integration points. The one API that will matter for real traversal but isn't exercised yet is `expand_one` with actual neighbors (currently it gets `std::iter::empty()`). The `run` method — which calls `expand_one` in a loop — is the eventual traversal entry point.

### Are `ef_search` defaults and bounds reasonable?

**Yes.** Verified in `mod.rs:23-25` and `options.rs:68-77`:
- Default: 40 (`TQHNSW_DEFAULT_EF_SEARCH`)
- Min: 1 (`TQHNSW_MIN_EF_SEARCH`)
- Max: 1000 (`TQHNSW_MAX_EF_SEARCH`)

These match standard HNSW implementations (hnswlib default ef_search=16, pgvector default 40). The min of 1 allows degenerate single-candidate scans for testing. The max of 1000 is reasonable given ADR-014's memory budget concerns — at 1000 candidates the visited set, candidate heap, and result buffer are bounded to manageable sizes.

Note: `ef_search` is registered as a reloption in `options.rs` and returned in `TqHnswOptions`, but the scan path still uses `MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3` (scan.rs:12). The reloption is groundwork — the actual wiring to pass `options.ef_search` into `reset_bootstrap_expansion_state` is a future slice.

### Is using the search helper for bootstrap expansion arbitration the right first integration seam?

**Yes.** The scan executor's `next_bootstrap_expand_index` (scan.rs:501-531) now creates a temporary `BeamSearch`, seeds unexpanded frontier candidates into it, and picks the best via `peek_best()`. This replaces the previous inline `min_by` scan. The `top_up_bootstrap_frontier` (scan.rs:592-616) uses `bootstrap_expansion_mut(opaque).expand_one(...)` to pop the best source from the scan-owned beam scheduler.

This is the right first integration: it proves the search API can drive scan-side decisions without changing visible behavior.

## Additional Findings

**One observation on `expand_one` usage:** In `top_up_bootstrap_frontier` (scan.rs:602-604), `expand_one` is called with `|_| std::iter::empty()` — the beam scheduler pops the best candidate but doesn't receive any neighbors to add. Instead, the scan-side `refill` callback handles neighbor discovery and calls `seed_discovered_candidates` (scan.rs:703→545-565) to push new candidates into both the frontier Vec and the beam scheduler. This means the beam scheduler and frontier Vec are maintained in parallel as dual data structures.

This dual-structure approach is correct for the current hybrid architecture but adds complexity. The eventual goal should be to make the beam scheduler the single source of truth, removing the frontier Vec. The `find_candidate_frontier_index` mapping (scan.rs:350-360) between beam nodes and Vec indices is the main coupling point that would need to be resolved.
