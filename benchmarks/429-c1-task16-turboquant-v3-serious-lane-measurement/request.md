# Review Request: C1 Task16 TurboQuant V3 Serious-Lane Measurement

Current head at execution: `735565a`

## Context

Packet `428` wired TurboQuant V3 hot/cold storage into the real AM lifecycle:

- build now writes `INDEX_FORMAT_V3_TURBO_HOT_COLD`
- insert / scan / vacuum now understand hot tuples plus cold rerank payloads
- TurboQuant no longer uses the old inline scalar element layout on current
  head

This packet measures what that actually means on the task-16 serious lane:

- isolated source-backed `50k, m=16` TurboQuant surface
- warm verified SQL latency at `ef_search = 128`
- matching internal TurboQuant stage profile
- recall summaries for both:
  - the recall-preserving `heap_f32` lane
  - the faster-but-lower-recall `quantized` override

## Environment

The old `/tmp/tqvector_pgrx_home` scratch lane no longer existed on disk, so
this packet recreated the scratch home and reloaded the minimal corpus needed
for task 16.

### Scratch bootstrap

1. copied `/home/peter/.pgrx/config.toml` into `/tmp/tqvector_pgrx_home`
2. installed current head with:

```bash
./scripts/install_adr030_pg17_pg_test.sh
```

3. started scratch with the task-16 heap-f32 lane settings:

```bash
./scripts/restart_adr030_scratch.sh \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32
```

4. created `tqvector` in the empty scratch database
5. loaded the canonical real-corpus tables from local staged TSVs:

```bash
./scripts/load_real_corpus_scratch.sh \
  --prefix tqhnsw_real_50k \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  --m 16 \
  --database postgres
```

### Isolated serious-lane surface

Then rebuilt the same one-index-per-table task-16 surface used in packets
`423` / `426`:

- corpus table: `tqhnsw_real_50k_turboquant_m16only_corpus`
- queries table: `tqhnsw_real_50k_turboquant_m16only_queries`
- index: `tqhnsw_real_50k_turboquant_m16only_m16_idx`

Created with:

```sql
CREATE TABLE tqhnsw_real_50k_turboquant_m16only_corpus
AS TABLE tqhnsw_real_50k_corpus;

CREATE TABLE tqhnsw_real_50k_turboquant_m16only_queries
AS TABLE tqhnsw_real_50k_queries;

CREATE INDEX tqhnsw_real_50k_turboquant_m16only_m16_idx
ON tqhnsw_real_50k_turboquant_m16only_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (
  m = 16,
  ef_construction = 128,
  build_source_column = 'source'
);
```

On current head that rebuild goes through the V3 TurboQuant writer.

## Commands

### Heap-f32 latency cell

```bash
./scripts/restart_adr030_scratch.sh \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32

./scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k_turboquant_m16only \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-v3-heapf32-m16only.summary
```

### Heap-f32 stage profile

Captured from `tests.tqhnsw_debug_turboquant_scan_stage_profile(...)` over the
same 50-query set into:

- `tmp/task16-turboquant-v3-heapf32-m16only-stageprofile.csv`

### Heap-f32 recall summary

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_turboquant_m16only_queries \
  --index tqhnsw_real_50k_turboquant_m16only_m16_idx \
  --corpus-table tqhnsw_real_50k_turboquant_m16only_corpus \
  --prefix tqhnsw_real_50k_turboquant_m16only
```

### Quantized comparison run on the same V3 index

Restarted scratch once more with:

```bash
./scripts/restart_adr030_scratch.sh \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode quantized
```

Then captured:

- `tmp/task16-turboquant-v3-quantized-m16only.summary`
- `tmp/task16-turboquant-v3-quantized-m16only-stageprofile.csv`
- a matching real-corpus recall summary on the same isolated V3 index

## Artifacts

- `tmp/task16-turboquant-v3-heapf32-m16only.summary`
- `tmp/task16-turboquant-v3-heapf32-m16only-stageprofile.csv`
- `tmp/task16-turboquant-v3-quantized-m16only.summary`
- `tmp/task16-turboquant-v3-quantized-m16only-stageprofile.csv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T174604Z_summary_tqhnsw_real_50k_turboquant_m16only_m16_idx_m16_ef128_tqhnsw_real_50k_turboquant_m16only_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T174837Z_summary_tqhnsw_real_50k_turboquant_m16only_m16_idx_m16_ef128_tqhnsw_real_50k_turboquant_m16only_queries.tsv`

## Results

### 1. V3 heap-f32 still preserves recall, but latency gets worse

Warm verified SQL latency on the isolated source-backed V3 lane:

- mean: `6.086ms`
- p50: `5.845ms`
- p95: `8.404ms`
- p99: `10.962ms`

Against packet `426`'s source-backed heap-f32 comparison point:

- previous heap-f32 mean: `5.220ms`
- current V3 heap-f32 mean: `6.086ms`
- delta: `+0.866ms` / `+16.59%`

Recall summary on the same V3 heap-f32 lane:

- `graph_recall_at_10 = 0.9629`
- `ndcg_at_10 = 0.9766229`
- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

That matches packet `426`'s heap-f32 recall readout rather than improving it.

### 2. The V3 heap-f32 regression lives in the rerank bucket, not traversal

Heap-f32 internal stage profile:

- hot `amrescan` mean: `2172.92us`
- traversal residual mean: `948.46us`
- binary-prefilter bucket mean: `1.54us`
- exact-score bucket mean: `1.04us`
- rerank bucket mean: `1221.88us`
- binary-prefilter survivors per query mean: `933.54`
- exact-score calls per query mean: `1.00`
- rerank calls per query mean: `40.00`
- exact-score mode: `mse_no_qjl_4bit`

Inference:

- the V3 layout did not make the recall-preserving path cheaper because the
  dominant cost is still the heap-f32 rerank fetch/comparison bucket
- traversal itself is not the bottleneck on this lane

### 3. V3 materially improves the quantized lane on the same rebuilt index

With the same V3 index but runtime override `--rerank-mode quantized`:

- mean: `2.158ms`
- p50: `2.145ms`
- p95: `2.778ms`
- p99: `2.806ms`

Against packet `426`'s quantized result:

- previous quantized mean: `2.958ms`
- current V3 quantized mean: `2.158ms`
- delta: `-0.800ms` / `-27.05%`

Quantized V3 stage profile:

- hot `amrescan` mean: `902.44us`
- traversal residual mean: `811.96us`
- binary-prefilter bucket mean: `3.66us`
- exact-score bucket mean: `41.36us`
- rerank bucket mean: `45.46us`
- exact-score calls per query mean: `41.00`
- rerank calls per query mean: `40.00`

Quantized recall summary stayed at packet `426`'s lower-recall point:

- `graph_recall_at_10 = 0.9251`
- `ndcg_at_10 = 0.9534868`
- `mean_abs_score_error = 0.006030937`
- `graph_below_exact_queries = 14`
- `worst_exact_gap = 1`

## Readout

### 1. V3 is directionally good for the fast quantized lane

The rebuilt V3 index improves the already-fast quantized override from
`2.958ms` down to `2.158ms` without changing the previously known recall
tradeoff (`0.9251` recall@10).

### 2. But V3 does not solve task 16's actual blocker

Task 16 needs the **recall-preserving** lane to be fast enough.
That lane is still heap-f32 rerank, and on V3 it gets worse:

- recall remains excellent (`0.9629`)
- latency regresses to `6.086ms`

### 3. The remaining problem is source-rerank cost, not TurboQuant traversal

The internal counters make the next step clear:

- traversal residual is under `1ms`
- heap-f32 rerank itself is ~`1.22ms`
- quantized rerank on the same index is only ~`45us`

Inference:

- the current V3 hot/cold split helps the quantized graph-hot path
- it does **not** address the expensive part of the recall-preserving lane,
  which is fetching/comparing raw source vectors during rerank

## Next Slice

The next justified task-16 change is not more TurboQuant tuple reshaping by
itself. It needs to attack the rerank source cost directly, for example by:

1. reducing heap-f32 rerank fetch cost, or
2. changing where the recall-preserving rerank payload lives, or
3. otherwise proving a new operating point for the heap-backed rerank lane

As measured here, packet `428` makes TurboQuant faster in the quantized lane
but does not close the serious-lane task-16 gap.
