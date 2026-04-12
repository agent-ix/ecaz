# Review Request: C1 ef_search=200 Planner Cost Crossover

## Context

Branch:
- `main`

Prior packets:
- `review/247-c1-real-corpus-latency-10k-verified-run/request.md`
- `review/249-c1-per-cell-planner-verification/request.md`

Packet `247` captured the first real `m=8` `10k` latency surface.

Packet `249` then proved the `~6295ms` `ef_search=200` line was a benchmark
integrity bug, not an HNSW runtime datapoint: once verified mode checked every
cell, the planner was shown to flip to `Sort -> Seq Scan` at `ef_search=200`.

The remaining problem was the planner crossover itself. On the live real
`10k` fixture before this patch:

- `SET tqhnsw.ef_search = 200; EXPLAIN ... LIMIT 10` chose `Sort -> Seq Scan`
- the seqscan+sort plan cost was about `1526.10`
- `SET enable_seqscan = off` on the same query still ran the tqhnsw index path
  in about `429.576ms`

So the live FR-020 CPU term was overcharging the index-side work enough to
abandon a materially faster ordered tqhnsw scan.

## Scope

- `src/am/cost.rs`
- `spec/functional/FR-020-cost-estimation.md`

## What Landed

### 1. Calibrated CPU term for LUT-scored tqhnsw work

`src/am/cost.rs` now applies a conservative CPU-dimension scale:

```text
LUT_CPU_DIMENSION_SCALE = 0.75
```

The cost model now charges:

- `graph_cpu = graph_pages * cpu_operator_cost * (dimensions * 0.75)`
- `linear_cpu = num_tuples * cpu_operator_cost * (dimensions * 0.75) * linear_fraction`

instead of using the raw dimension count directly.

The reasoning is specific to tqhnsw’s ordered scan path: the hot scoring work
is LUT-backed code scoring, not full raw-f32 arithmetic at every candidate.
Using the raw dimension count was conservative enough to be wrong for the real
`1536`-dimension `ef_search=200` LIMIT-10 probe.

### 2. Added a live-like crossover regression to the pure cost model

`src/am/cost.rs` now includes a unit test with the observed real-`10k`
metadata:

- `index_pages = 1251`
- `reltuples = 10000`
- `m = 8`
- `ef_search = 200`
- `dimensions = 1536`
- `tree_height = 4`

The test asserts that the modeled startup cost stays below the observed
seqscan+sort crossover (`1526.10`) so the planner does not fall off the index
for this exact C1 shape again.

### 3. Updated FR-020 to match the live implementation

`spec/functional/FR-020-cost-estimation.md` is now aligned with the actual D2
implementation:

- `graph_pages = tree_height + ef_search`
- residual linear work is scaled by `linear_fraction`
- the CPU term uses the calibrated LUT-scoring dimension factor

This also corrects the stale older formula that still described the superseded
`tree_height * m + ef_search * 2 * m` scaffolding.

## Validation

Targeted validation:

- `cargo test am::cost::tests::`
- `cargo pgrx test pg17 test_fr020_ac1_planner_chooses_index_scan_for_large_table`

Representative live C1 checks after install:

- `SET tqhnsw.ef_search = 200; SELECT modeled_startup_cost, modeled_total_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_real_10k_m8_idx'::regclass);`
  - `1403.52`, `26554.11712230216`
- `SET tqhnsw.ef_search = 200; EXPLAIN ...`
  - planner now chooses `Index Scan using tqhnsw_real_10k_m8_idx`
- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 200 --query-limit 1 --output /tmp/tqv_verified_guard_ef200.summary`
  - succeeds and records:
    `m=8 ef_search=200 n=1 mean=413.156ms`

Required checkpoint validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All green.

## Current Status

At this checkpoint:

- the representative real-`10k` `m=8, ef_search=200` query is back on the
  tqhnsw index without disabling seqscan
- the per-cell verified launcher from packet `249` now permits an `ef=200`
  smoke run instead of aborting on planner fallback
- the full `200`-query rerun for `m=8, ef_search=200` plus the `m=16` sweep
  are still outstanding in packet `247`

So this checkpoint fixes the planner crossover, but it does **not** by itself
close C1.

## Review Focus

- Is `0.75` the right conservative calibration boundary for the LUT-backed CPU
  term, given that it restores the real `1536`-dim `ef=200` crossover while
  preserving the small-table seqscan preference test?
- Is the new live-like unit test the right permanent guard for this regression,
  or should the crossover assertion live at the pg-test layer instead?
- Is the FR-020 spec now aligned closely enough with the active D2
  implementation, or does the document need more explicit language about the
  empirical nature of the CPU calibration?
