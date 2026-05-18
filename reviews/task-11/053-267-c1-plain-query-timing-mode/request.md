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

## Implementation

Completed work:

1. Added `--timing-mode explain|plain-server` to
   `scripts/bench_sql_latency.sh`.
2. Kept the existing per-cell planner verification unchanged, so the verified
   launcher still aborts before timing if the wrong index or a seqscan plan is
   selected.
3. Added a `plain-server` branch that times the ordered query with
   `clock_timestamp()` around a `MATERIALIZED` subquery, while preserving the
   existing warmup and session reuse controls.
4. Updated `docs/RECALL_REAL_CORPUS.md` and
   `spec/non-functional/NFR-001-query-latency.md` so the new timing seam is
   documented and the spec no longer implies that `EXPLAIN` timing is the only
   reporting surface.

## Result

Current local read on the warm verified `10K`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell` seam:

```text
explain mode:      p50=11.024ms p95=13.244ms p99=15.491ms mean=11.111ms
plain-server mode: p50=10.932ms p95=13.377ms p99=16.915ms mean=11.020ms
```

So far this does **not** support the earlier assumption that `EXPLAIN`
instrumentation is the dominant reason the warm C1 surface still sits around
`11ms`. The new mode is useful, but the current result says the bottleneck is
somewhere else.

## Validation

- `bash -n scripts/bench_sql_latency.sh` passed
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All required gates were rerun and completed green after the final script state.

## Conclusion

This slice is worth keeping because it gives the C1 lane an honest
planner-verified plain-query timing surface. The important finding, though, is
that plain server-side timing is almost identical to the explain-based warm
surface on the current `10K` fixture. That means the next C1 slice should stop
chasing `EXPLAIN` overhead and instead investigate statement planning / query
submission overhead or another warm-path seam outside the scorer microbench.
