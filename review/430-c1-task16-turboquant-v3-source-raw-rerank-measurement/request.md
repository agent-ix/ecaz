# Review Request: C1 Task16 TurboQuant V3 Source-Raw Rerank Measurement

Current head at execution: `fd42eb5`

## Context

Packet `429` established two points on current head:

- the recall-preserving V3 `heap_f32` lane regressed to `6.086ms`
- the dominant cost on that lane was still the heap rerank bucket

This packet probes the next obvious no-code experiment already exposed by
current head:

- keep the same isolated `50k, m=16` V3 index
- add a packed `bytea` raw-f32 heap source column
- switch the runtime rerank source via
  `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN=source_raw`
- measure whether that makes the serious lane cheaper without changing recall

No repo code changed in this slice. This is a scratch measurement packet only.

## Setup

Starting from packet `429`'s isolated task-16 surface:

- corpus table: `tqhnsw_real_50k_turboquant_m16only_corpus`
- queries table: `tqhnsw_real_50k_turboquant_m16only_queries`
- index: `tqhnsw_real_50k_turboquant_m16only_m16_idx`

### Materialize `source_raw`

Added and backfilled a packed raw-f32 column with the existing test helper:

```sql
ALTER TABLE tqhnsw_real_50k_turboquant_m16only_corpus
ADD COLUMN IF NOT EXISTS source_raw bytea;

UPDATE tqhnsw_real_50k_turboquant_m16only_corpus
SET source_raw = tests.tqhnsw_debug_pack_f32_bytea(source)
WHERE source_raw IS NULL;
```

That produced `6144` bytes per row on average, which matches `1536 * 4`.

### Important runtime finding: the old index was no longer usable

Running the packet-`429` verified SQL cell immediately after that full-table
update failed with repeated errors like:

```text
tqhnsw PqFastScan heap rerank source vector could not fetch heap tuple at (4274,3)
```

So the table-wide heap rewrite left the existing HNSW entries pointing at stale
heap TIDs for this AM. The isolated index had to be rebuilt before any
meaningful measurement could continue:

```sql
REINDEX INDEX tqhnsw_real_50k_turboquant_m16only_m16_idx;
```

## Commands

### Rebuilt `source_raw` serious-lane run

```bash
./scripts/restart_adr030_scratch.sh \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32 \
  --rerank-source-column source_raw

./scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k_turboquant_m16only \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-v3-heapf32-source-raw-m16only.summary
```

Repeated once into:

- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only-rerun.summary`

### `source_raw` stage profile

Captured the same packet-`429` aggregate helper into:

- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only-stageprofile.csv`

### `source_raw` recall summary

```bash
./scripts/run_real_corpus_recall_scratch.sh summary \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_turboquant_m16only_queries \
  --index tqhnsw_real_50k_turboquant_m16only_m16_idx \
  --corpus-table tqhnsw_real_50k_turboquant_m16only_corpus \
  --prefix tqhnsw_real_50k_turboquant_m16only
```

### Same-table source-vs-`source_raw` comparison

To isolate the source-column effect from the rebuild itself, restarted scratch
back to the default heap-f32 lane on the same rebuilt table:

```bash
./scripts/restart_adr030_scratch.sh \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32
```

Then captured:

- `tmp/task16-turboquant-v3-heapf32-source-same-table-m16only.summary`
- `tmp/task16-turboquant-v3-heapf32-source-same-table-m16only-rerank-profile.csv`
- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only-rerank-profile.csv`

Those rerank-profile CSVs use the more detailed existing helper
`tests.tqhnsw_debug_grouped_rerank_profile(...)`.

## Artifacts

- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only.summary`
- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only-rerun.summary`
- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only-stageprofile.csv`
- `tmp/task16-turboquant-v3-heapf32-source-raw-m16only-rerank-profile.csv`
- `tmp/task16-turboquant-v3-heapf32-source-same-table-m16only.summary`
- `tmp/task16-turboquant-v3-heapf32-source-same-table-m16only-rerank-profile.csv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T180934Z_summary_tqhnsw_real_50k_turboquant_m16only_m16_idx_m16_ef128_tqhnsw_real_50k_turboquant_m16only_queries.tsv`

## Results

### 1. Rebuilt `source_raw` heap-f32 preserves the serious-lane recall point

Warm verified SQL on the rebuilt `source_raw` lane:

- first run mean: `4.568ms`
- first run p50 / p95 / p99: `4.489ms / 5.287ms / 5.673ms`
- rerun mean: `4.593ms`
- rerun p50 / p95 / p99: `4.543ms / 5.256ms / 5.667ms`

Recall summary on the same rebuilt `source_raw` lane:

- `graph_recall_at_10 = 0.9629`
- `ndcg_at_10 = 0.9766229`
- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

That exactly matches packet `429`'s recall-preserving heap-f32 readout.

### 2. The headline gain versus packet `429` is real, but not all of it is the `source_raw` override

Against packet `429`'s source-backed V3 heap-f32 lane:

- previous mean: `6.086ms`
- rebuilt `source_raw` mean: `4.568ms`
- delta: `-1.518ms` / `-24.94%`

Against packet `426`'s earlier source-backed serious lane:

- previous mean: `5.220ms`
- rebuilt `source_raw` mean: `4.568ms`
- delta: `-0.652ms` / `-12.49%`

But the fair same-table comparison on the rebuilt surface is smaller:

- rebuilt same-table `source` mean: `4.889ms`
- rebuilt same-table `source_raw` mean: `4.568ms`
- delta: `-0.321ms` / `-6.57%`

So `source_raw` is a real win, but most of the full `429 -> 430` gap is not
explained by the source-column swap alone.

### 3. The detailed rerank helper shows where `source_raw` helps

On the same rebuilt table and same index:

| metric | `source` | `source_raw` | delta |
|---|---:|---:|---:|
| rerank total mean | `1495.52us` | `1290.08us` | `-205.44us` |
| heap fetch mean | `111.52us` | `42.18us` | `-69.34us` |
| heap decode mean | `1297.68us` | `1161.94us` | `-135.74us` |
| heap dot mean | `40.42us` | `42.48us` | `+2.06us` |

Shared across both modes:

- heap rerank calls mean: `40.00`
- heap rows fetched mean: `40.00`

Inference:

- the win is not in the dot product itself
- the win comes from cheaper heap fetch/decode on the packed raw-f32 source
- the existing coarse stage helper was too blunt to make that comparison
  obvious without the same-table rerank-profile split

### 4. The old index/TID failure is a real rollout constraint

The first post-backfill benchmark attempt was not just slower; it was broken
until `REINDEX INDEX ...` repaired the stale heap TIDs.

That means a raw-source-column rollout is not just "add a better source and
point the runtime at it". It also needs explicit update/reindex/maintenance
semantics for existing indexes on already-populated tables.

## Readout

### 1. A packed raw-f32 heap source is the first current-head task-16 lever that actually helps the serious lane

This is the first task-16 probe on current head that:

- keeps the recall-preserving heap-f32 correctness point
- beats packet `429` materially
- also beats packet `426`'s earlier source-backed serious-lane latency

### 2. The next justified implementation is a supported raw-source path, not more TurboQuant tuple surgery

Packet `429` already showed V3 tuple layout work did not fix the serious lane.
This packet shows the serious-lane cost moves when the heap rerank source
representation changes.

So the next high-value productizable step is:

- a durable raw-f32 source column/path for heap-f32 rerank
- with explicit rebuild/update safety semantics

not another round of TurboQuant hot/cold tuple reshaping by itself.
