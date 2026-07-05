# Review Request: C1 Beam Search Lazy Queued Removal

## Context

Packet `268` closed the remaining launcher-side timing seam question for the
warm verified `10K`, `m=8`, `ef_search=40` run:

- `explain`: `mean=11.111ms`
- `plain-server`: `mean=11.020ms`
- `cached-plan`: `mean=11.041ms`

That result was important but negative. Once the query runs in a warm
persistent backend, neither `EXPLAIN` output nor fresh statement planning is
the dominant remaining gap. The next slice should therefore return to the
executor / AM hot path.

Packet `264` flagged one specific warm-path churn seam in `src/am/search.rs`:
`BeamSearch::forget_queued` currently drains the whole `BinaryHeap`, filters
out one node, collects a temporary `Vec`, and rebuilds the heap.

## Problem

In the current `BeamSearch` implementation:

- `peek_best_matching` repeatedly calls `forget_queued` when the scheduler head
  is stale relative to the visible frontier
- `take_best_matching` does the same when consuming the first live scheduler
  node
- each `forget_queued` call is an O(n) heap drain + rebuild

At `ef_search=40` the heap is small, but this path runs in the steady-state
result-emission loop and compounds with repeated stale-leader drops. At higher
`ef_search` values it gets worse.

## Implementation

Completed work:

1. Replaced the eager heap drain in `src/am/search.rs` with a separate live
   queued-node set.
2. `forget_queued` now clears queued state lazily instead of draining and
   rebuilding the full `BinaryHeap`.
3. `peek_best`, `pop_best`, `snapshot_frontier`, `frontier_len`, and `is_empty`
   now respect the live queued set so stale leaders are skipped at the head and
   do not leak back through scheduler inspection.
4. Kept the existing scheduler semantics for reseeding, visited accounting, and
   stale-leader dropping in the search tests.

## Result

Current local read on the warm verified `10K`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell`, `cached-plan` seam:

```text
before: p50=11.028ms p95=13.461ms p99=14.857ms mean=11.041ms
after:  p50=10.932ms p95=13.137ms p99=15.059ms mean=10.993ms
```

That is only a small move, but it is at least directionally positive on p50 /
mean / p95. This is not a C1-closing win; it is a low-signal cleanup checkpoint
that removes a clearly avoidable heap rebuild path without regressing the warm
surface.

## Validation

- targeted scheduler tests:
  - `cargo test beam_search_`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`,
  `cached-plan`

All required gates were rerun and completed green after the final search state.

## Conclusion

This slice is worth keeping because it removes an obviously wasteful
`BinaryHeap` drain/rebuild path and leaves the warm verified surface slightly
better, not worse. The effect is small enough that the next C1 slice should aim
at a larger remaining churn source rather than spend more time polishing the
scheduler.
