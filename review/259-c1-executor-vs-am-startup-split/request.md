# Review Request: C1 Executor vs AM Startup Split

## Context

Packet `258` added a finer-grained startup probe for the tqhnsw ordered-scan
 path and recorded the first real breakdown on the canonical real-`10k` lane.

For query `id=10000` on `tqhnsw_real_10k_m8_idx`, the new
`tests.tqhnsw_debug_scan_hot_path_profile(...)` data shows only about:

- `ef_search=40`
  - initialize entry: `5.630ms`
  - candidate scoring: `3.476ms`
  - graph element load: `0.696ms`
  - graph neighbor load: `0.250ms`
- `ef_search=200`
  - initialize entry: `23.099ms`
  - candidate scoring: `13.302ms`
  - graph element load: `2.962ms`
  - graph neighbor load: `0.978ms`

But the real SQL probe for the same `m=8 / ef_search=40 / id=10000` case still
reports:

- `Index Scan using tqhnsw_real_10k_m8_idx`
  - startup: `46.187ms`
  - total: `52.091ms`

So packet `258` established that the current AM-local counters are real, but
they still do not explain the node startup wall time seen by PostgreSQL.

## Problem

The next optimization target is ambiguous until the missing startup time is
placed on the right side of the AM boundary.

If the remaining time is still inside tqhnsw startup, the next slice should add
another AM-local probe or fix. If the time is mostly heap/executor work after
the AM has already chosen and staged TIDs, then continuing to optimize the AM
hot path will have sharply diminishing returns.

## Planned work

1. Compare AM-local startup timing against a heap-only fetch of the already
   selected top-`k` TIDs for the same real-`10k` query.
2. Use that comparison to decide whether the next C1 slice belongs in tqhnsw
   startup or in executor-visible row fetch costs.
3. Record the result in this packet with a concrete recommendation for the next
   optimization seam.

## Exit criteria

- this packet identifies whether the remaining `~40ms` startup gap is primarily
  inside tqhnsw AM startup or outside it
- the result is grounded in a representative real-`10k` probe, not inference
  from counters alone
- the next optimization target becomes narrower than the current “somewhere in
  index scan startup” state
