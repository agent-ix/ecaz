# Review Request: C1 Plain Query Timing Mode

## Context

Packet `265` improved the verified warm `10K`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell` surface to about `11.11ms mean`.

Packet `266` then recorded a failed AVX2 scorer probe: direct scorer microbench
did not beat the existing scalar no-QJL `1536x4-bit` path, and the verified
warm SQL cell remained essentially flat.

At this point the remaining gap is more likely measurement overhead than raw
scorer CPU. The current launcher still derives every latency cell from
per-query `EXPLAIN (ANALYZE, FORMAT JSON)` execution times.

## Problem

In `scripts/bench_sql_latency.sh`:

- the planner verification step is good and should stay
- but the only timing mode is still `EXPLAIN (ANALYZE)` per query
- that means the warm C1 surface includes explain/instrumentation overhead even
  when the user really wants plain ordered-query latency

So the current warm verified surface is still not a clean read of steady-state
query latency.

## Planned work

1. Add a plain server-side query timing mode alongside the current explain mode.
2. Keep planner verification active before timing each measured cell.
3. Preserve the existing explain-based mode for compatibility and debugging.
4. Record the warm per-cell `10K`, `m=8`, `ef_search=40` result with the new
   plain timing seam.

## Exit criteria

- verified launcher still aborts when the planner picks the wrong index
- benchmark script can report plain server-side query timings without relying
  on per-query `EXPLAIN (ANALYZE)`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- the new plain timing read is recorded for the warm `10K`, `m=8`,
  `ef_search=40`, `warm-after-prime3`, `per-cell` seam
