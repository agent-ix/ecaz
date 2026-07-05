# Review Request: C1 Task16 Persisted Rerank-Source Measurement

Current head at execution: `239238a`

## Context

Packet `431` turned packet `430`'s useful `source_raw` experiment into a real
`pq_fastscan` reloption:

- `WITH (rerank_source_column = 'source_raw')`
- validated at build / empty-build time
- used by the default grouped rerank resolution when no env override is present

This packet measures that persisted path directly on the isolated task-16
serious lane and verifies that the runtime is actually using the persisted
default rather than an env-forced rerank mode.

No repo code changed in this slice. This is a measurement-only packet on top of
`239238a`.

## Setup

### 1. Reused the isolated task-16 tables

The existing isolated surface from packets `429` / `430` was still present:

- corpus table: `tqhnsw_real_50k_turboquant_m16only_corpus`
- queries table: `tqhnsw_real_50k_turboquant_m16only_queries`
- index name: `tqhnsw_real_50k_turboquant_m16only_m16_idx`

### 2. Installed current head into the scratch lane

```bash
./scripts/install_adr030_pg17_pg_test.sh
```

### 3. Rebuilt the isolated index with a persisted rerank source

The current-head reloption requires `storage_format = 'pq_fastscan'`, so the
final rebuilt index definition was:

```sql
DROP INDEX tqhnsw_real_50k_turboquant_m16only_m16_idx;

CREATE INDEX tqhnsw_real_50k_turboquant_m16only_m16_idx
ON tqhnsw_real_50k_turboquant_m16only_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (
  m = 16,
  ef_construction = 128,
  storage_format = 'pq_fastscan',
  build_source_column = 'source',
  rerank_source_column = 'source_raw'
);
```

### 4. Important measurement caveat: the stock scratch restart helper forces a rerank-mode env override

`./scripts/restart_adr030_scratch.sh` always exports
`TQVECTOR_PQ_FASTSCAN_RERANK_MODE`, even when the requested mode matches the
current Rust default.

That means using the helper directly would have produced:

- `pq_fastscan_rerank_mode = heap_f32`
- `pq_fastscan_rerank_mode_resolution = env_override`

which is the wrong lane for this packet.

So the final scratch start for measurement was a manual start with only:

- `TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW=64`
- `TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE=binary`

and **without** `TQVECTOR_PQ_FASTSCAN_RERANK_MODE` or
`TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN`.

### 5. Runtime confirmation on the measured lane

On the live measured server, the debug runtime settings probe returned:

```text
heap_f32    default_heap_f32_with_rerank_source_column    source_raw
```

So the measured lane really was using the persisted rerank-source default.

## Commands

### Warm verified SQL latency

Ran three warm verified cells on the same persisted-source lane:

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k_turboquant_m16only \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-v3-persisted-source-raw-m16only.summary
```

Repeated into:

- `tmp/task16-turboquant-v3-persisted-source-raw-m16only-rerun.summary`
- `tmp/task16-turboquant-v3-persisted-source-raw-m16only-rerun2.summary`

### Recall summary

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_turboquant_m16only_queries \
  --index tqhnsw_real_50k_turboquant_m16only_m16_idx \
  --corpus-table tqhnsw_real_50k_turboquant_m16only_corpus \
  --prefix tqhnsw_real_50k_turboquant_m16only
```

## Artifacts

- `tmp/task16-turboquant-v3-persisted-source-raw-m16only.summary`
- `tmp/task16-turboquant-v3-persisted-source-raw-m16only-rerun.summary`
- `tmp/task16-turboquant-v3-persisted-source-raw-m16only-rerun2.summary`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T185834Z_summary_tqhnsw_real_50k_turboquant_m16only_m16_idx_m16_ef128_tqhnsw_real_50k_turboquant_m16only_queries.tsv`

## Results

### 1. The persisted reloption really does drive the default heap-f32 rerank lane

The live runtime settings row on the measured server was:

- rerank mode: `heap_f32`
- rerank mode resolution: `default_heap_f32_with_rerank_source_column`
- rerank source column: `source_raw`

So this packet is not re-measuring the old env-override path from `430`.

### 2. Recall still holds on the persisted-source lane

Recall summary on the rebuilt persisted-source index:

- `graph_recall_at_10 = 0.9635`
- `graph_recall_at_100 = 0.76542`
- `ndcg_at_10 = 0.976963`
- `mean_abs_score_error = 0`
- `spearman_rho_at_10 = 0.92972225`
- `exact_quantized_recall_at_10 = 0.9078`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

That preserves the serious-lane exact-score point.

### 3. Latency on this restart was materially noisier than packet `430`

Three warm verified SQL runs on the same lane:

- run 1 mean: `5.386ms`
- run 1 p50 / p95 / p99: `5.340ms / 7.002ms / 7.208ms`
- run 2 mean: `6.361ms`
- run 2 p50 / p95 / p99: `5.907ms / 10.016ms / 11.996ms`
- run 3 mean: `4.649ms`
- run 3 p50 / p95 / p99: `4.639ms / 5.457ms / 5.645ms`

Observed spread:

- min mean: `4.649ms`
- max mean: `6.361ms`
- median-of-three mean: `5.386ms`

Against packet `430`'s env-override `source_raw` point:

- packet `430` first run mean: `4.568ms`
- packet `430` rerun mean: `4.593ms`

So this packet does **not** reproduce packet `430`'s tighter pair of
`~4.58ms` runs on this scratch restart, even though the best run here
(`4.649ms`) is still close to that earlier point.

## Readout

### 1. Packet `431`'s reloption works as intended

The AM can now reach the source-raw heap-f32 lane through persisted index
metadata alone. The measured runtime settings prove that the default path
selected `heap_f32` from `rerank_source_column` without any rerank env override.

### 2. The correctness point survives productization

The persisted-source lane still delivers exact-score behavior:

- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

### 3. The performance readout is currently "same lane, noisier restart"

This packet confirms the persisted path can hit the same basic latency regime as
packet `430`, but it does not support a tighter "persisted is equal to env
override" claim yet because the three-run spread was much wider on this restart.

### 4. The restart helper currently hides this lane unless started manually

For future persisted-default measurement, `scripts/restart_adr030_scratch.sh`
is misleading because it always forces `TQVECTOR_PQ_FASTSCAN_RERANK_MODE`.

That helper behavior is now the main friction point for re-measuring the true
persisted-default lane.
