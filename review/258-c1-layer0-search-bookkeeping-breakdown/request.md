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

## Checkpoint Findings

The first instrumentation pass landed and the scratch SQL wrapper was refreshed
so the new hot-path columns are queryable against the real `10k` fixture.

Representative `tests.tqhnsw_debug_scan_hot_path_profile(...)` output on query
`id=10000` now shows:

- `ef_search=40`
  - prepare query: `0us`
  - reset state: `4us`
  - initialize entry: `5630us`
  - upper-layer seed: `1116us`
  - layer-0 seed: `4429us`
  - stage ordered results: `51us`
  - initial prefetch: `18us`
  - frontier consume: `14us`
  - graph result materialize: `2us`
  - graph element load: `696us` across `241` misses
  - graph neighbor load: `250us` across `46` misses
  - candidate scoring: `3476us` across `241` misses and `205` cache hits
- `ef_search=200`
  - prepare query: `0us`
  - reset state: `6us`
  - initialize entry: `23099us`
  - upper-layer seed: `965us`
  - layer-0 seed: `21861us`
  - stage ordered results: `221us`
  - initial prefetch: `38us`
  - frontier consume: `32us`
  - graph result materialize: `3us`
  - graph element load: `2962us` across `997` misses
  - graph neighbor load: `978us` across `202` misses
  - candidate scoring: `13302us` across `997` misses and `1156` cache hits

Those numbers are materially useful, but they do **not** explain the real SQL
startup latency by themselves. A representative SQL probe for the same
`m=8 / ef_search=40 / id=10000` query:

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
SELECT id
FROM tqhnsw_real_10k_corpus
ORDER BY embedding <#> (SELECT source FROM tqhnsw_real_10k_queries WHERE id = 10000)
LIMIT 10;
```

still reports:

- `Index Scan using tqhnsw_real_10k_m8_idx`
  - startup: `46.187ms`
  - total: `52.091ms`

So this packet closes one ambiguity and opens the next one more sharply:
the current seed/load/score counters are real, but the remaining node startup
cost is still outside the buckets currently recorded in `opaque.debug_profile`.
The next profiling pass needs to reconcile that SQL node time against the AM
startup boundary instead of assuming the remaining gap is all inside layer-0
bookkeeping.

## Exit criteria

- this packet records where the remaining rescan time actually goes
- the slice either lands one targeted layer-0 bookkeeping optimization or
  leaves a concrete profile that justifies the next code change
- any landed code still clears the standard checkpoint gate before push
