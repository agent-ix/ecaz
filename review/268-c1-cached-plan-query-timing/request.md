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

## Planned work

1. Add a cached-plan timing mode for `per-cell` runs.
2. Use a server-side cached query surface so repeated queries in the cell reuse
   the same planned ordered scan shape.
3. Keep the existing planner verification before timing the cell.
4. Record the same warm verified `10K`, `m=8`, `ef_search=40` cell against the
   cached-plan seam.

## Exit criteria

- new cached-plan timing mode only runs when session reuse makes it meaningful
- verified launcher still aborts when the representative plan is wrong
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`
  read recorded for the cached-plan seam
