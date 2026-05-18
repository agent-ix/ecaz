# Review Request: C1 ADR-030 V2 Scratch Explicit-Format Runtime Matrix

Current head at execution: `095014c`

## Context

The branch already had:

- current-head explicit-format `50k` gate proof in packet `405`
- current-head `50k m=16 ef=128` full-summary proof in packet `407`

But it still did not have one compact, same-runtime-lane matrix across:

- corpus sizes: `10k`, `50k`
- storage formats: `turboquant`, `pq_fastscan`
- `m`: `8`, `16`
- `ef_search`: `40,64,100,128,160,200`

That is the exact read this packet captures.

## Environment

I avoided the live `~/.pgrx` socket path and reran on the approved `/tmp`
scratch cluster only:

1. stopped the `~/.pgrx` postmaster that had been holding port `28817`
2. restarted the approved scratch cluster with:
   - `./scripts/restart_adr030_scratch.sh --window 64 --grouped-score-mode binary`
3. loaded explicit-format families into scratch with:
   - `./scripts/load_real_corpus_scratch.sh ... --storage-format turboquant`
   - `./scripts/load_real_corpus_scratch.sh ... --storage-format pq_fastscan`
4. used the extension-owned direct runtime sweep:

```sql
select * from tests.tqhnsw_graph_scan_recall_ef_sweep(
  <corpus_table>,
  <queries_table>,
  <index_name>,
  <m>,
  array[40,64,100,128,160,200]
);
```

All runs below therefore share one runtime lane:

- `pq_fastscan` traversal score mode: `binary`
- `pq_fastscan` rerank mode: `heap_f32`
- `pq_fastscan` rerank source column: `build_source_column`
- scratch cluster only

## Results

The returned columns are:

| col | meaning |
|-----|---------|
| 1 | `m` |
| 2 | `ef_search` |
| 3 | `Recall@10` |
| 4 | `exact_quantized Recall@10` |
| 5 | `mean abs score error` |
| 6 | `mean query latency ms` |

### 10k `turboquant`, `m=8`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.9355` | `0.9660` | `0.8532` |
| 64  | `0.9445` | `0.9660` | `1.1625` |
| 100 | `0.9455` | `0.9660` | `1.5965` |
| 128 | `0.9460` | `0.9660` | `1.8987` |
| 160 | `0.9465` | `0.9660` | `2.2429` |
| 200 | `0.9465` | `0.9660` | `2.5806` |

### 10k `turboquant`, `m=16`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.9660` | `0.9660` | `0.9516` |
| 64  | `0.9690` | `0.9660` | `1.2966` |
| 100 | `0.9700` | `0.9660` | `1.7525` |
| 128 | `0.9710` | `0.9660` | `2.0967` |
| 160 | `0.9720` | `0.9660` | `2.4200` |
| 200 | `0.9720` | `0.9660` | `2.8444` |

### 10k `pq_fastscan`, `m=8`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.9550` | `0.9660` | `1.6466` |
| 64  | `0.9590` | `0.9660` | `2.4844` |
| 100 | `0.9615` | `0.9660` | `3.7855` |
| 128 | `0.9620` | `0.9660` | `4.6364` |
| 160 | `0.9625` | `0.9660` | `5.6838` |
| 200 | `0.9635` | `0.9660` | `7.2350` |

### 10k `pq_fastscan`, `m=16`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.9790` | `0.9660` | `1.6890` |
| 64  | `0.9840` | `0.9660` | `2.5366` |
| 100 | `0.9885` | `0.9660` | `3.7738` |
| 128 | `0.9900` | `0.9660` | `4.7761` |
| 160 | `0.9900` | `0.9660` | `5.8985` |
| 200 | `0.9900` | `0.9660` | `7.3617` |

### 50k `turboquant`, `m=8`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.8301` | `0.8301` | `1.3563` |
| 64  | `0.8645` | `0.8301` | `1.9630` |
| 100 | `0.8854` | `0.8301` | `2.7639` |
| 128 | `0.8927` | `0.8301` | `3.3244` |
| 160 | `0.8966` | `0.8301` | `3.9214` |
| 200 | `0.9011` | `0.8301` | `5.3834` |

### 50k `turboquant`, `m=16`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.9123` | `0.8301` | `2.0121` |
| 64  | `0.9253` | `0.8301` | `2.8327` |
| 100 | `0.9324` | `0.8301` | `4.0410` |
| 128 | `0.9342` | `0.8301` | `5.2543` |
| 160 | `0.9366` | `0.8301` | `5.9217` |
| 200 | `0.9376` | `0.8301` | `7.1441` |

### 50k `pq_fastscan`, `m=8`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.8231` | `0.8301` | `1.9491` |
| 64  | `0.8705` | `0.8301` | `3.0370` |
| 100 | `0.8976` | `0.8301` | `4.6524` |
| 128 | `0.9078` | `0.8301` | `5.9426` |
| 160 | `0.9129` | `0.8301` | `7.9919` |
| 200 | `0.9174` | `0.8301` | `8.4302` |

### 50k `pq_fastscan`, `m=16`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean latency ms |
|----------:|----------:|--------------------------:|----------------:|
| 40  | `0.9078` | `0.8301` | `2.2913` |
| 64  | `0.9468` | `0.8301` | `3.4720` |
| 100 | `0.9616` | `0.8301` | `5.2667` |
| 128 | `0.9635` | `0.8301` | `7.0190` |
| 160 | `0.9657` | `0.8301` | `8.3110` |
| 200 | `0.9671` | `0.8301` | `9.8369` |

## Readout

The matrix is materially different from the earlier branch story.

### 1. `pq_fastscan` wins top-10 recall everywhere that matters

At every serious operating point in this scratch rerun:

- `10k`
  - `m=8, ef=128`: `0.9620` vs `0.9460`
  - `m=16, ef=128`: `0.9900` vs `0.9710`
- `50k`
  - `m=8, ef=128`: `0.9078` vs `0.8927`
  - `m=16, ef=128`: `0.9635` vs `0.9342`

The spread grows with both corpus size and `m`.

### 2. On this runtime surface, `pq_fastscan` is also slower

That recall gain is not free on the direct runtime lane.

Representative same-cell comparisons:

- `10k, m=8, ef=128`
  - `turboquant`: `0.9460 @ 1.8987ms`
  - `pq_fastscan`: `0.9620 @ 4.6364ms`
- `10k, m=16, ef=128`
  - `turboquant`: `0.9710 @ 2.0967ms`
  - `pq_fastscan`: `0.9900 @ 4.7761ms`
- `50k, m=8, ef=128`
  - `turboquant`: `0.8927 @ 3.3244ms`
  - `pq_fastscan`: `0.9078 @ 5.9426ms`
- `50k, m=16, ef=128`
  - `turboquant`: `0.9342 @ 5.2543ms`
  - `pq_fastscan`: `0.9635 @ 7.0190ms`

So the current default source-backed heap-rerank `pq_fastscan` lane has become
the higher-recall path, not the lower-latency path.

### 3. `m=16` is the strong production-shaped operating area

The user asked specifically about `m=16`, and the rerun confirms it is the
real operating area:

- `10k, pq_fastscan, m=16`
  - already at `0.9790` by `ef=40`
  - reaches `0.9900` by `ef=128`
- `50k, pq_fastscan, m=16`
  - already at `0.9468` by `ef=64`
  - reaches `0.9635` by `ef=128`
  - reaches `0.9671` by `ef=200`

That is the strongest current branch evidence for landing `pq_fastscan` as a
first-class format on `main`.

## SQL Timing Note

I also started the planner-verified SQL latency lane with:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_10k \
  --storage-format turboquant \
  --m 8 \
  --ef-search 40,64,100,128,160,200 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan
```

The launcher behaved correctly but exposed an important planner fact:

- on the shared `10k` table, requesting `m=8` at `ef=40` planned on the
  `m=16` index instead
- requesting `m=16` later in the same corpus can flip back onto the `m=8`
  index at higher `ef`

So a **clean full SQL matrix on the shared tables is not honest** without
either:

1. isolated one-index-per-table benchmark surfaces, or
2. planner work that makes per-`m` index choice stable

That is why this packet records the fully executed **direct runtime** matrix
and does not overclaim a shared-table SQL matrix that the verified launcher
itself refused to bless.

## Outcome

This batch answers the open landing question directly.

1. yes, the final current-head scratch lane has now been rerun across:
   - `10k`
   - `50k`
   - `turboquant`
   - `pq_fastscan`
   - `m=8`
   - `m=16`
   - `ef=40,64,100,128,160,200`
2. `pq_fastscan` is currently the higher-recall format across the serious
   operating points
3. on this direct runtime surface, that higher recall costs more latency than
   `turboquant`
4. the missing measurement gap is now narrowed to the planner-facing SQL lane,
   not the direct runtime recall/timing surface

## Next Slice

The next honest measurement slice is not “rerun the same direct matrix again.”
It is one of:

1. create isolated one-index-per-table SQL benchmark surfaces for the explicit
   `m=8` / `m=16` families, or
2. investigate why the planner cross-chooses between `m=8` and `m=16` on the
   shared tables before claiming a shared-table SQL spectrum
