# Feedback: Localized Frontier Node Lookup

Request:
- `review/90-localized-frontier-node-lookup.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-06

## Response to Review Focus

### Is removing the shared node-to-index helper the right cleanup?

**Yes.** `find_candidate_frontier_index` is gone from the codebase. Its two former consumers now handle node lookup differently:
1. **Visibility checks**: `candidate_frontier_contains` (scan.rs:433-438) delegates to `VisibleCandidateFrontierRef::contains_node` (scan.rs:352-355), which does a direct `any()` check — no index needed
2. **Removal**: `take_candidate_frontier_node` (scan.rs:714-718) delegates to `VisibleCandidateFrontier::remove_node` (scan.rs:406-411), which does `position()` + `remove()` internally — index is a private container detail

The shared helper was the last place that exported a Vec-index as a reusable concept across multiple scan paths. Removing it means no external code can reason about "the index of node X in the frontier." This is the right encapsulation.

### Do any remaining scan paths promote scheduler-owned identity back into reusable Vec-index semantics?

**No.** All paths now work in TID space:
- `scheduler_best_frontier_node` → returns TID
- `current_candidate_frontier_head_tid` → returns TID
- `consume_candidate_frontier_head` → takes by TID
- `candidate_frontier_contains` → checks by TID
- `seed_discovered_candidates` → seeds by candidate (which carries TID)

The only index operations are internal to `VisibleCandidateFrontier`:
- `remove_node` — `position()` + `remove()` (write path)
- `slot(index)` — positional access for debug (read path)

Neither leaks index semantics to callers.

### Should the next slice target the visible frontier container itself?

**Yes.** With the shared index helper gone, the remaining Vec-specific behavior is fully encapsulated behind `VisibleCandidateFrontier` / `VisibleCandidateFrontierRef`. The container is ready to be treated as an opaque type. Reviews 91-95 build out this encapsulation, which is the natural next step before ownership transfer.

## Additional Findings

No issues found. Clean removal of the last shared index abstraction.
