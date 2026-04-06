# Feedback: Two-Slot Frontier Head Ordering

Request:
- `review/47-two-slot-frontier-head-ordering.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is `valid first, then lower score` the right ordering rule?

**Yes.** The `recompute_candidate_frontier_head` implementation (scan.rs:529-549) skips candidates with `!score_valid` then selects the lowest score among valid candidates, breaking ties by lower index. This correctly implements the `<#>` convention where lower = more similar. The tie-break on index is arbitrary but deterministic, which is what matters at this stage.

One observation: the function iterates the entire Vec on every recomputation. This is fine for small frontiers but will need to be replaced when the Vec grows to `MAX_BOOTSTRAP_FRONTIER_CANDIDATES` size. The current call pattern — recompute after every push/remove — means this is O(n) per frontier mutation. For the bootstrap frontier sizes involved this is negligible, but worth noting as a future optimization point when profiling.

### Should the frontier head be explicit state or lazily derived?

**Explicit state is the right call.** Storing `candidate_frontier_head: Option<usize>` in `TqScanOpaque` avoids recomputing on every access and makes the "no valid head" state explicit (via `None`). The alternative — deriving lazily from the Vec — would either require a scan on every head access or push the ordering logic into every caller. The current approach centralizes ordering in `recompute_candidate_frontier_head` and lets consumers just read the stored index.

The `Option<usize>` representation is clean — `None` means no valid candidate, `Some(i)` is the best slot index. This directly maps to what `consume_candidate_frontier_head` needs.

### Missing lifecycle edges around head recomputation?

No gaps found. The head is recomputed in all the right places:
- `clear_scan_candidate_state` (scan.rs:292-298) clears the Vec and sets head to `None`
- `recompute_candidate_frontier_head` is called after every frontier mutation in `refill_candidate_frontier_from_source` (scan.rs:570, 577, 613)
- `consume_candidate_frontier_head` (scan.rs:560) recomputes after removing the consumed slot
- `reset_scan_position` calls `clear_scan_candidate_state` which handles rescan

No path mutates the frontier Vec without a subsequent head recomputation.

## Additional Findings

No issues found. The ordering rule is minimal and correct for this stage.
