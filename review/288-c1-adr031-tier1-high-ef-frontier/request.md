# Review Request: C1 ADR-031 Tier 1 High-EF Frontier

## Context

Packet `287` established that the ADR-031 Tier 1 inline cache slice is a major
warm-latency win on the real `50k` canonical seam at:

- `m=8`
- `ef_search=40`
- `1000` queries
- `warm-after-prime3`
- `session-mode=per-cell`
- `timing-mode=cached-plan`

with repeated release reads around:

- `p50 ~= 1.48ms`
- `p99 ~= 2.4ms`
- `mean ~= 1.51ms`

and a current-build recall recheck showing no runtime regression versus exact
quantized results on that same `ef_search=40` seam.

The next question is not whether ADR-031 works at all. It does. The next
question is where the current latency/recall frontier sits at the higher
`ef_search` settings that matter for apples-to-apples comparison with the older
A4 gate work.

## Problem

The old A4 evidence and the current ADR-031 Tier 1 evidence are easy to
miscompare because they use different seams:

- A4 closeout centered on `ef_search=128` and smaller real-query slices
- packet `287` centered on `ef_search=40` and the full canonical `1000`-query
  table

We need two explicit reads on the current Tier 1 build:

1. canonical current-build recall + latency at `ef_search=128` and `200`
2. a same-query-table apples-to-apples read against the old A4 `queries_50`
   surface so the comparison does not mix query counts or `ef_search`

## Planned Investigation

On the current Tier 1 build:

1. Run full real-`50k` canonical recall summaries at:
   - `m=8`, `ef_search=128`
   - `m=8`, `ef_search=200`
2. Run full real-`50k` warm latency summaries at the same two points.
3. Reuse the historical `tqhnsw_real_50k_queries_50` table and record the
   current `m=8` high-`ef_search` recall there as the apples-to-apples A4
   comparison seam.

## Success Criteria

- the packet records current-build real-`50k` recall at `ef_search=128` and
  `200`
- the packet records current-build real-`50k` warm latency at `ef_search=128`
  and `200`
- the packet records a same-query-table comparison against the old
  `queries_50` A4 surface
- the packet makes a clear call on whether Tier 2 should be next or whether the
  high-`ef_search` frontier still needs work first
