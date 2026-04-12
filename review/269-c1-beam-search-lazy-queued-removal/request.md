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

## Planned work

1. Change queued-node removal to a lazy scheme instead of draining and
   rebuilding the heap every time.
2. Preserve the current scheduler semantics:
   stale queued nodes disappear from `peek_best_matching`, `take_best_matching`,
   `snapshot_frontier`, and reseeding rules.
3. Add or update unit tests around queued removal and stale-leader skipping.
4. Rerun the full validation gate plus the warm verified `10K`, `m=8`,
   `ef_search=40`, `warm-after-prime3`, `per-cell` seam.

## Exit criteria

- queued-node removal no longer drains and rebuilds the full heap
- scheduler semantics remain unchanged for the existing search tests
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`
  read recorded
