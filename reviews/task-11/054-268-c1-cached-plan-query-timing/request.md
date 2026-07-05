# Review Request: C1 Cached-Plan Query Timing

## Context

Packet `267` added a planner-verified `plain-server` timing seam to
`scripts/bench_sql_latency.sh`.

That result was important but negative: on the warm verified `10K`, `m=8`,
`ef_search=40`, `warm-after-prime3`, `per-cell` seam, `plain-server` came back
essentially identical to the existing explain-based timing surface.

## Problem

The current launcher now supports:

- `explain`: per-query `EXPLAIN (ANALYZE, FORMAT JSON)`
- `plain-server`: plain ordered query timed with `clock_timestamp()`

But both modes still submit a fresh SQL statement for every timed query. That
means the warm read can still include parse / planning overhead for the query
text itself even when the AM and index data are already warm.

Packet `264`'s survey makes this boundary worth closing quickly before another
AM-focused checkpoint: if cached plan reuse moves the warm read materially,
there is still harness-side churn to peel away; if it stays flat, the next
slice should go back to executor / AM hot-path work instead of more timing
seams.

## Implementation

Completed work:

1. Added `--timing-mode cached-plan` to `scripts/bench_sql_latency.sh`.
2. Restricted that mode to `--session-mode per-cell`, since session reuse is
   what makes server-side plan reuse meaningful.
3. Added a per-cell `pg_temp.tqv_latency_cached_plan(real[])` plpgsql helper
   that times the ordered query with `clock_timestamp()` while reusing the same
   planned ordered-scan statement across the full cell.
4. Kept the existing planner verification unchanged, so the verified launcher
   still aborts before timing if the representative plan does not select the
   expected tqhnsw index.
5. Updated `docs/RECALL_REAL_CORPUS.md` and
   `spec/non-functional/NFR-001-query-latency.md` so the new seam is
   documented explicitly.

## Result

Current local read on the warm verified `10K`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell` seam:

```text
explain mode:      p50=11.024ms p95=13.244ms p99=15.491ms mean=11.111ms
plain-server mode: p50=10.932ms p95=13.377ms p99=16.915ms mean=11.020ms
cached-plan mode:  p50=11.028ms p95=13.461ms p99=14.857ms mean=11.041ms
```

This is another negative-but-useful result: cached plan reuse does **not** move
the warm C1 mean materially. The warm surface is still about `11ms`, so parse /
planning churn is not the dominant remaining gap on this `10K` lane.

## Validation

- `bash -n scripts/bench_sql_latency.sh` passed
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All required gates were rerun and completed green after the final script state.

## Conclusion

This slice is worth keeping because it closes the remaining obvious timing-seam
question from packets `264` and `267`. The important finding is still negative:
once the query runs in a warm persistent backend, neither `EXPLAIN` output nor
fresh statement planning explains the remaining `~11ms` C1 latency. The next
checkpoint should return to executor / AM hot-path work rather than more
launcher-side timing variants.
