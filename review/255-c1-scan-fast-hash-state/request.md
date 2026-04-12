# Review Request: C1 Scan Fast Hash State

## Context

Packet `254` landed the scan-local score cache and materially improved the real
`10k` latency surface. Packet `247` now records the repaired score-cache rerun:

- canonical `m=8, ef_search=40`: mean `89.089ms`
- canonical `m=8, ef_search=200`: mean `173.680ms`

That is a real win, but it still misses `NFR-001` badly.

The post-score-cache hot-path probe on the same representative real query
(`id=10000`) now shows a different bottleneck shape:

- `ef_search=40`
  - upper-layer seed elapsed: `24.560ms`
  - layer-0 seed elapsed: `14.301ms`
  - candidate scoring elapsed: `31.711ms`
- `ef_search=200`
  - upper-layer seed elapsed: `65.931ms`
  - layer-0 seed elapsed: `69.229ms`
  - candidate scoring elapsed: `110.260ms`

So traversal bookkeeping is now in the same band as scoring, and the hot path
is still heavily keyed by `ItemPointer` lookups:

- scan-local visited / expanded / emitted sets
- scan-local graph and score caches
- beam-search visited sets during seed search

## Problem

The ordered scan hot path still uses `std` `HashMap` / `HashSet` with their
default hasher across the main `ItemPointer` bookkeeping surfaces. After the
score-cache win, those structures are more likely to matter:

- `search_layer0_result_candidates_with_successors(...)` constructs a fresh
  visited set during rescan seeding
- `BeamSearch` tracks visited nodes during traversal and refill work
- scan-local caches and state sets are consulted repeatedly during the same
  query

This is a narrow, plausible next optimization target that does not change the
search algorithm or planner behavior.

## Planned work

1. Switch the scan/search hot-path `ItemPointer` map/set structures to a faster
   hash implementation.
2. Keep the slice narrow:
   - no algorithm rewrite
   - no planner changes
   - no harness changes
3. Re-run the hot-path probe and representative real-fixture latency surface to
   verify whether the bookkeeping cost drops materially.

## Exit criteria

- a pushed checkpoint narrows the hash-heavy scan/search bookkeeping cost on
  the real C1 path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- this packet records measured before/after evidence, not just the code change
