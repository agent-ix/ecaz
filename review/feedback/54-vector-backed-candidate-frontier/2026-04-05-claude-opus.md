# Feedback: Vector-Backed Candidate Frontier

Request:
- `review/54-vector-backed-candidate-frontier.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is a heap-owned `Vec<ScanCandidate>` the right next representation?

**Yes.** The `Vec` replaces the fixed two-slot array while keeping the same semantic contract: an ordered collection of scored candidates with explicit head tracking. A `Vec` naturally supports the upcoming growth needs (wider seeding, frontier expansion from neighbor exploration) without the fixed-width limitations of inline storage.

The ownership model (`Box::into_raw` / `Box::from_raw` on the Vec itself, stored as `*mut Vec<ScanCandidate>` in scan.rs:960) is consistent with visited set and prepared query ownership. The accessor helpers `candidate_frontier_ref` / `candidate_frontier_mut` (scan.rs:309-321) centralize the null-check and dereference, which is clean.

### Lifecycle or compaction edge cases around head removal?

`consume_candidate_frontier_head` (scan.rs:552-561) uses `Vec::remove(head)` which shifts elements left, then recomputes the head. This is O(n) for the shift, which is fine for small frontiers. When the frontier grows to `MAX_BOOTSTRAP_FRONTIER_CANDIDATES` or beyond, the shift cost could matter — but at that point the Vec will likely be replaced by a `BinaryHeap` where consumption is O(log n) via `pop()`.

One edge worth noting: `Vec::remove` on the head index is correct because `recompute_candidate_frontier_head` is called immediately after, which rescans and updates the index. There's no window where a stale index could be consumed. The bounds check at scan.rs:555 (`head >= len()`) provides an additional safety net.

`clear_scan_candidate_state` (scan.rs:292-298) correctly handles the Vec lifecycle — if null, allocates; otherwise clears. This is called from both rescan and exhaustion paths. No lifecycle gap.

### Should the frontier-head representation change before traversal expansion?

**The current `Option<usize>` is fine.** It will naturally become unnecessary when the Vec is replaced by a `BinaryHeap` (head = `peek()`). Until then, the explicit index into the Vec is the simplest correct representation. No reason to change it before the container itself changes.

## Additional Findings

No issues found. The transition from fixed slots to Vec is clean and doesn't change observable behavior.
