# Feedback: Frontier Head Consumption

Request:
- `review/49-frontier-head-consumption.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is it correct for head consumption to return the consumed slot while leaving the other slots untouched?

**Yes.** `consume_candidate_frontier_head` (scan.rs:552-561) removes only the head slot via `Vec::remove(head)`, which shifts subsequent elements left, then recomputes the head from whatever remains. This is exactly right — consumption should only affect the consumed element. The remaining candidates are still valid exploration targets.

The `Vec::remove` shift means indices change after consumption, which is why recomputation is mandatory. The implementation handles this correctly by calling `recompute_candidate_frontier_head` immediately after the remove.

### Is recomputing from the remaining valid slot sufficient groundwork?

**Yes.** The recomputation scans all remaining candidates and picks the best valid one. This is the correct minimal behavior — it doesn't assume anything about frontier size or ordering invariants beyond "pick the lowest-scoring valid candidate." When a `BinaryHeap` replaces the Vec, `recompute` becomes `peek()` and `consume` becomes `pop()`, but the semantic contract is identical.

### Missing edge cases around empty frontier or double-consumption?

The unit test (`consume_candidate_frontier_head_reselects_then_clears`, scan.rs:2285-2360) covers:
- Consuming the best slot, verifying the remaining slot is reselected
- Consuming the last slot, verifying head becomes `None`
- Consuming from an empty frontier, verifying `None` is returned

One subtle edge worth noting: `consume_candidate_frontier_head` checks `head >= candidate_frontier_ref(opaque).len()` (scan.rs:555) as a safety guard against a stale head index pointing past the end. This could only happen if something mutated the Vec without recomputing the head — a programming error, not a runtime scenario. The guard is appropriate defensive code.

**No missing edge cases.** The empty-frontier and double-consumption paths are explicitly tested.

## Additional Findings

No correctness issues. The consume-then-recompute pattern is clean and will translate directly to a heap-based frontier.
