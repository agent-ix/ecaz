# Review Request: C1 Task16 TurboQuant Quantized Default Measurement

Current head at execution: `c9a90bf`

## Context

Packet `425` changed source-backed turboquant to default to deferred quantized
rerank while keeping `heap_f32` available as an explicit override.

This packet measures what that policy means on the task-16 serious lane:

- isolated one-index-per-table `50k, m=16` source-backed turboquant surface
- warm verified SQL cell at `ef_search = 128`
- matching internal stage profile
- isolated recall summaries for both quantized and heap-f32 rerank on the same
  surface

## Environment

- scratch pg17 cluster at `/tmp/tqvector_pgrx_home`
- corpus table: `tqhnsw_real_50k_turboquant_m16only_corpus`
- queries table: `tqhnsw_real_50k_turboquant_m16only_queries`
- index: `tqhnsw_real_50k_turboquant_m16only_m16_idx`
- planner lane: verified one-index-per-table surface, same shape as packet
  `423`

Important lane note:

- `scripts/restart_adr030_scratch.sh` still defaults
  `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=heap_f32`
- the SQL latency + stage-profile run therefore passed `--rerank-mode
  quantized` explicitly to measure the code path that current-head pg tests now
  prove is turboquant's source-backed default
- the comparison recall rerun used `--rerank-mode heap_f32` on the same
  isolated surface

## Commands

Install current head into scratch:

```bash
./scripts/install_adr030_pg17_pg_test.sh
```

Quantized latency cell:

```bash
./scripts/restart_adr030_scratch.sh \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode quantized

./scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k_turboquant_m16only \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-quantized-default-m16only.summary
```

Quantized internal stage profile:

```sql
COPY (
  WITH queries AS (
    SELECT source
    FROM tqhnsw_real_50k_turboquant_m16only_queries
    ORDER BY id
    LIMIT 50
  ),
  profile AS (
    SELECT p.*
    FROM queries q
    CROSS JOIN LATERAL tests.tqhnsw_debug_turboquant_scan_stage_profile(
      'tqhnsw_real_50k_turboquant_m16only_m16_idx'::regclass::oid,
      q.source
    ) AS p
  )
  SELECT
    round(avg(rescan_amrescan_total_elapsed_us)::numeric, 2) AS rescan_mean_us,
    round(avg(turboquant_traversal_residual_elapsed_us)::numeric, 2) AS traversal_residual_mean_us,
    round(avg(turboquant_binary_prefilter_score_calls)::numeric, 2) AS binary_prefilter_calls_mean,
    round(avg(turboquant_binary_prefilter_score_elapsed_us)::numeric, 2) AS binary_prefilter_elapsed_mean_us,
    round(avg(turboquant_binary_prefilter_survivor_candidates)::numeric, 2) AS binary_prefilter_survivors_mean,
    round(avg(turboquant_exact_score_calls)::numeric, 2) AS exact_score_calls_mean,
    round(avg(turboquant_exact_score_elapsed_us)::numeric, 2) AS exact_score_elapsed_mean_us,
    round(avg(turboquant_rerank_score_calls)::numeric, 2) AS rerank_score_calls_mean,
    round(avg(turboquant_rerank_score_elapsed_us)::numeric, 2) AS rerank_score_elapsed_mean_us,
    min(turboquant_exact_score_mode) AS exact_score_mode,
    bool_or(turboquant_exact_score_uses_lut) AS any_exact_score_uses_lut,
    bool_or(turboquant_exact_score_uses_qjl) AS any_exact_score_uses_qjl
  FROM profile
) TO STDOUT WITH CSV HEADER;
```

Recall comparison on the same isolated surface:

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_turboquant_m16only_queries \
  --index tqhnsw_real_50k_turboquant_m16only_m16_idx \
  --corpus-table tqhnsw_real_50k_turboquant_m16only_corpus \
  --prefix tqhnsw_real_50k_turboquant_m16only
```

with scratch restarted once for quantized and once for `heap_f32`.

## Artifacts

- `tmp/task16-turboquant-quantized-default-m16only.summary`
- `tmp/task16-turboquant-quantized-default-m16only-stageprofile.csv`
- `tmp/task16-turboquant-quantized-default-m16only.txt`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T151140Z_summary_tqhnsw_real_50k_turboquant_m16only_m16_idx_m16_ef128_tqhnsw_real_50k_turboquant_m16only_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T151338Z_summary_tqhnsw_real_50k_turboquant_m16only_m16_idx_m16_ef128_tqhnsw_real_50k_turboquant_m16only_queries.tsv`

## Results

### Quantized deferred-rerank latency

On the isolated warm verified SQL cell:

- mean: `2.958ms`
- p50: `2.887ms`
- p95: `3.724ms`
- p99: `4.252ms`

Against packet `423`'s baseline on the same isolated surface:

- previous mean: `5.046ms`
- current quantized mean: `2.958ms`
- delta: `-2.088ms` / `-41.38%`

Against the task-15 `pq_fastscan` reference cell:

- `pq_fastscan`: `4.26ms`
- turboquant quantized on isolated source-backed surface: `2.958ms`

### Quantized internal stage profile

The hot-path shape now looks like a real deferred-rerank pipeline instead of
binary-prefilter traversal exact-scoring every survivor:

- hot `amrescan` mean: `1.06760ms`
- traversal residual mean: `0.98258ms`
- binary-prefilter bucket mean: `0.00074ms`
- exact-score bucket mean: `0.04176ms`
- rerank bucket mean: `0.04252ms`
- binary-prefilter survivors per query mean: `933.54`
- exact-score calls per query mean: `41.00`
- rerank calls per query mean: `40.00`
- exact-score mode: `mse_no_qjl_4bit`
- exact-score `uses_lut`: `false`
- exact-score `uses_qjl`: `false`

Packet `423` baseline on the same surface was:

- hot `amrescan` mean: `4.75400ms`
- traversal residual mean: `3.04980ms`
- exact-score bucket mean: `1.69838ms`
- exact-score calls per query mean: `1605.52`
- rerank calls per query mean: `0`

### Isolated recall tradeoff

On the same isolated source-backed surface at `m=16`, `ef_search=128`,
`1000` queries:

- quantized rerank:
  - `graph_recall_at_10 = 0.9251`
  - `ndcg_at_10 = 0.9534868`
  - `mean_abs_score_error = 0.006030937`
  - `graph_below_exact_queries = 14`
  - `worst_exact_gap = 1`
- heap-f32 rerank:
  - `graph_recall_at_10 = 0.9629`
  - `ndcg_at_10 = 0.9766229`
  - `mean_abs_score_error = 0`
  - `graph_below_exact_queries = 0`
  - `worst_exact_gap = 0`

The earlier heap-f32 latency readout on this same isolated surface from packet
`424` was `5.220ms` mean SQL latency.

## Readout

This is the task-16 decision point.

### 1. Deferred quantized rerank closes the latency gap

For pure SQL latency, packet `425` puts turboquant in a much better place:

- `2.958ms` mean on the isolated warm lane
- `~41` quantized exact comparisons/query instead of `~1605`

### 2. But it does not hold the serious recall target on the source-backed lane

The isolated recall summary shows the cost of that latency win:

- quantized rerank: `0.9251` recall@10
- heap-f32 rerank: `0.9629` recall@10

So quantized turboquant is fast here, but not at the recall target that the
task asks to preserve.

### 3. Heap-f32 restores recall parity but is still too slow

Heap-f32 on the same isolated source-backed lane lands at essentially recall
parity (`0.9629`) but packet `424` measured it at `5.220ms`, still behind
`pq_fastscan`'s `4.26ms` reference cell.

### 4. Decision

Levers `1` and `2` are not enough to satisfy task 16 at the same recall target:

- quantized deferred rerank wins latency but misses recall
- heap-f32 rerank wins recall but misses latency

The next justified implementation step is lever `3`: hot/cold turboquant
payload split, aimed at making the recall-preserving heap-f32 path fast enough
to compete with `pq_fastscan` without giving up the source-backed exact rerank.
