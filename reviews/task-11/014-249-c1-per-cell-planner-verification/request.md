# Review Request: C1 Per-Cell Planner Verification

## Context

Branch:
- `main`

Prior packets:
- `review/246-c1-latency-launcher-plan-verification/request.md`
- `review/247-c1-real-corpus-latency-10k-verified-run/request.md`

Packet `246` added the first verified launcher so C1 would stop timing obvious
`Sort -> Seq Scan` plans.

Packet `247` then captured the first real `m=8` `10k` surface and showed a
dramatic `ef_search=200` cliff (`~6295ms` mean). Follow-up on the exact query
shape showed that this was not an HNSW runtime collapse. It was a launcher
integrity bug:

- the old verified launcher only checked one optimistic EXPLAIN before the run
- later cells reused the same sweep without re-checking planner choice
- at `ef_search=200`, the planner flipped to `Sort -> Seq Scan`
- with `enable_seqscan = off`, the same representative query still ran as an
  index scan in `~429.576ms`

So the benchmark surface needed a stricter guarantee: verified mode must check
the actual plan for each measured `(m, ef_search)` cell before timing it.

## Scope

- `scripts/bench_sql_latency.sh`
- `scripts/bench_sql_latency_verified.sh`
- `scripts/tests/test_bench_sql_latency_verified.py`

## What Landed

### 1. Per-cell verification moved into the real-corpus delegate

`scripts/bench_sql_latency.sh` now accepts an internal expected-index contract
via `TQV_REQUIRE_INDEX_NAME`.

When that env var is present, each real-corpus cell now:

- resolves the first probe query from `<prefix>_queries`
- runs `SET tqhnsw.ef_search = <ef>; EXPLAIN ...`
- refuses to time the cell unless the plan still uses the exact expected
  tqhnsw index

If the planner falls back to `Sort -> Seq Scan`, the script aborts before any
per-query `EXPLAIN (ANALYZE)` timings are collected for that cell.

### 2. The verified wrapper now exports the expected index instead of doing one optimistic preflight

`scripts/bench_sql_latency_verified.sh` still enforces the one-`m`-per-run
contract, but it now delegates verification to the real timing loop by
exporting:

```text
TQV_REQUIRE_INDEX_NAME=<prefix>_m{N}_idx
```

That makes verified mode match the real measured query shape instead of
checking only a single pre-run EXPLAIN.

### 3. New shell regression covers later-cell fallback

`scripts/tests/test_bench_sql_latency_verified.py` drives the real shell
launchers against a fake `psql` binary and proves two cases:

- verified mode aborts when a later `ef_search` cell falls back from the index
- verified mode succeeds when every cell stays on the expected index

This gives cheap regression coverage for the benchmark harness without needing
a live Postgres cluster.

## Validation

Shell / harness validation:

- `bash -n scripts/bench_sql_latency.sh`
- `bash -n scripts/bench_sql_latency_verified.sh`
- `bash -n scripts/bench_sql_latency_verified_scratch.sh`
- `python3 scripts/tests/test_bench_sql_latency_verified.py`

Real smoke against the scratch cluster:

- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 40 --query-limit 1 --output /tmp/tqv_verified_guard_ok.summary`
  - passes and records one valid indexed cell
- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 200 --query-limit 1 --output /tmp/tqv_verified_guard_fail.summary`
  - aborts before timing and prints the `Sort -> Seq Scan` plan

Required checkpoint validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Note: the repo-local `/tmp/tqvector_pgrx_home` was absent during this
checkpoint, so `cargo pgrx test pg17` was validated against the configured
default `PGRX_HOME` (`/home/peter/.pgrx`) instead.

All green.

## Current Status

At this checkpoint:

- verified C1 latency runs can no longer silently record later-cell planner
  fallbacks
- packet `247` has been corrected so the old `ef_search=200` `~6295ms` line is
  treated as invalid
- the remaining blocker is the planner-cost crossover itself, because the live
  planner still chooses `Seq Scan + Sort` at `ef_search=200` on the real `10k`
  surface even though the forced index path is much faster

## Review Focus

- Is `TQV_REQUIRE_INDEX_NAME` the right minimal contract for verified-mode
  delegation, or should the index verification live behind an explicit CLI flag
  instead?
- Is “first non-empty query row from `<prefix>_queries`” a reasonable probe for
  per-cell verification, given that the failure mode we are guarding is a
  planner crossover on the benchmark query shape itself?
- Is the new fake-`psql` regression the right maintenance boundary for shell
  harness behavior, or should this move under a broader script-test harness
  later?
