# Review Request: C1 Layer-0 Search Bookkeeping Breakdown

## Context

Packet `257` landed the QJL-disabled 4-bit score fast path and moved the best
verified canonical real-`10k` `m=8` surface on `main` to:

- `ef_search=40`: mean `50.521ms`
- `ef_search=200`: mean `68.260ms`

That is a real C1 step-change, but it also changed the bottleneck mix.

Representative rescan profile on query `id=10000` now reports:

- `ef_search=40`
  - `rescan_elapsed_us = 42046`
  - `emit_elapsed_us = 221`
  - `total_elapsed_us = 42315`
- `ef_search=200`
  - `rescan_elapsed_us = 63370`
  - `emit_elapsed_us = 2599`
  - `total_elapsed_us = 66121`

So ordered-scan runtime is still overwhelmingly dominated by `amrescan`, not
by output emission.

The current hot-path probe for the same query reports:

- `ef_search=40`
  - upper-layer seed elapsed: `1.223ms`
  - layer-0 seed elapsed: `5.145ms`
  - graph element load elapsed: `1.547ms`
  - graph neighbor load elapsed: `0.290ms`
  - candidate score elapsed: `3.240ms`
- `ef_search=200`
  - upper-layer seed elapsed: `1.186ms`
  - layer-0 seed elapsed: `23.698ms`
  - graph element load elapsed: `4.486ms`
  - graph neighbor load elapsed: `1.119ms`
  - candidate score elapsed: `13.555ms`

Those tracked buckets do not explain the full rescan wall time, especially at
`ef_search=40`. The remaining time is likely in layer-0 search bookkeeping:
heap operations, visited-set maintenance, and result-window management inside
`search_layer0_result_candidates_with_successors`.

## Problem

The graph I/O and score-path wins have removed the easy first-order bottlenecks.
The remaining gap to `NFR-001` appears to be dominated by traversal machinery
rather than candidate math.

Right now that is only an informed suspicion. The next slice needs to make that
internal cost visible enough to optimize surgically instead of guessing.

## Planned work

1. Instrument the layer-0 result-window search path so the bookkeeping inside
   `search_layer0_result_candidates_with_successors` becomes measurable.
2. Separate search bookkeeping time from neighbor loading and scoring time on
   the real `10k` lane.
3. If the data points to a clear low-risk structural win, implement it in the
   same slice; otherwise stop with a concrete, reviewable profile of the next
   hotspot.

## Exit criteria

- this packet records where the remaining rescan time actually goes
- the slice either lands one targeted layer-0 bookkeeping optimization or
  leaves a concrete profile that justifies the next code change
- any landed code still clears the standard checkpoint gate before push
