# Review Request: C1 Warm-Cache Verified Surface

## Context

Packet `260` changed the C1 interpretation materially:

- representative warm-cache SQL startup on the real `10k` corpus is now around
  `4.1ms` at `m=8, ef_search=40`
- repeated plain scans of that same representative query are around
  `1.1ms/query`
- the large remaining gap on the current verified surface is cold-cache I/O,
  not hidden warm-path CPU overhead

That means the current C1 reporting is incomplete relative to `NFR-001`, which
already requires warm-cache and cold-cache results to be reported separately
when feasible.

## Problem

The current verified launcher and durable C1 artifacts still center the cold
`EXPLAIN`-timed surface. That makes the C1 read misleading now that the warm
surface appears to be at or below the NFR target on the representative `10k`
lane.

## Planned work

1. Add a verified warm-cache measurement seam that preserves the existing
   planner/index guard.
2. Capture a representative warm-cache result on the real `10k` `m=8` lane.
3. Report warm and cold separately in the C1 packet and status/docs.

## In-Progress Findings

- The original shell harness opens a fresh `psql` connection for every timed
  query. That means a naive warmup pass does not actually preserve backend-local
  state for the timed query set.
- This packet now adds two explicit real-corpus controls:
  - `--warmup-passes N`
  - `--session-mode per-query|per-cell`
- The verified planner/index guard still runs before any warmup or timing.

Representative `tqhnsw_real_10k`, `m=8`, `ef_search=40` results so far:

- `session-mode=per-query`, `warmup-passes=1`
  - `p50=50.246ms`
  - `p95=53.903ms`
  - `p99=57.319ms`
  - `mean=50.496ms`
- `session-mode=per-cell`, `warmup-passes=1`
  - `p50=15.883ms`
  - `p95=24.149ms`
  - `p99=32.691ms`
  - `mean=16.843ms`
- `session-mode=per-cell`, `warmup-passes=3`
  - `p50=14.315ms`
  - `p95=16.350ms`
  - `p99=17.613ms`
  - `mean=14.194ms`
- Representative single-query warm spot check (`query-limit=1`,
  `session-mode=per-cell`, `warmup-passes=1`)
  - `p50=12.488ms`

## Current Read

Backend/session reuse materially changes the measured warm surface, but it does
not make the current `10k` lane pass `NFR-001`.

The strongest honest warm reading captured through the committed verified seam
so far is still about `p50=14.3ms` on the `10k` real-corpus lane, which is
well above the `p50 < 5ms` target and still on the smaller-than-normative
table size.

So packet `260` was still directionally right that session behavior matters,
but the broader warm-cache C1 read is not “already solved.” The next C1 work
should treat warm steady-state latency as improved but still failing, and
should target the remaining query-set-wide cost rather than relying on a single
representative fast query.

The current profile-backed next seam is now clearer too: after correcting the
warm measurement surface, the strongest code-level suspects are no longer the
launcher itself but the remaining scan/runtime copy boundaries in graph tuple
read/decode and scan-result materialization.

## Checkpoint

- Code checkpoint: `c1832d4` `bench: add verified warm per-cell latency mode`
- Validation:
  - `python3 scripts/tests/test_bench_sql_latency_verified.py`
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Packet status: open

This checkpoint closes the benchmark-seam part of the warm-cache question:
verified warm runs are now reproducible and no longer accidentally measure
fresh-backend churn as if it were steady-state query latency.

## Exit criteria

- warm-cache measurement is reproducible through a committed repo-local seam
- the warm run still refuses to measure the wrong planner/index path
- C1 reporting clearly distinguishes warm vs cold instead of treating the cold
  surface as the only headline
