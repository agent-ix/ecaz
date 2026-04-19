# Review Request: C1 Task16 TurboQuant Persisted-Source Supported Measurement

Current head at execution: `6007c69`

## Context

Packet `439` productized `rerank_source_column` for TurboQuant. The remaining
task-16 question was the supported-path measurement:

- same TurboQuant serious lane
- same index/table
- no runtime source-column env override
- use persisted `rerank_source_column = source_raw`
- compare against the same index with the reloption reset back to
  `build_source_column`

This packet measures that supported path on current head.

## Important setup note

An initial scratch run produced flat `~26.96ms` numbers and could not see the
TurboQuant stage-profile helper. That turned out to be a bad measurement
surface: the scratch cluster had not been reinstalled on current head yet, so
it was still running an older `tqvector.so`.

That stale-install run is **not** used for the readout below.

The authoritative run in this packet starts only after:

```bash
./scripts/install_adr030_pg17_pg_test.sh --pgrx-home /home/peter/.pgrx

./scripts/restart_adr030_scratch.sh \
  --pgrx-home /home/peter/.pgrx \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32
```

## Measurement surface

Built one fresh comparison corpus up front from the canonical 50k lane so the
same heap tuples back both source modes:

```sql
CREATE TABLE tqhnsw_real_50k_tq_srccompare_corpus AS
SELECT
  id,
  source,
  tests.tqhnsw_debug_pack_f32_bytea(source) AS source_raw,
  embedding
FROM tqhnsw_real_50k_corpus;

ANALYZE tqhnsw_real_50k_tq_srccompare_corpus;
```

Built one TurboQuant index:

```sql
CREATE INDEX tqhnsw_real_50k_tq_srccompare_src_m16_idx
ON tqhnsw_real_50k_tq_srccompare_corpus
USING tqhnsw (embedding)
WITH (
  m = 16,
  ef_construction = 128,
  build_source_column = source,
  storage_format = turboquant
);
```

Then measured by flipping only:

```sql
ALTER INDEX tqhnsw_real_50k_tq_srccompare_src_m16_idx
RESET (rerank_source_column);

ALTER INDEX tqhnsw_real_50k_tq_srccompare_src_m16_idx
SET (rerank_source_column = source_raw);
```

So the fair comparison is:

- same current-head install
- same table
- same index name
- same backend settings
- same `heap_f32` serious lane
- only the persisted rerank-source selector changes

## Commands

### Baseline source-backed serious lane

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_tq_srccompare \
  --corpus-table tqhnsw_real_50k_tq_srccompare_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_tq_srccompare_src_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-persisted-source-source-current.summary
```

Repeated at `--query-limit 200` into:

- `tmp/task16-turboquant-persisted-source-source-current-q200.summary`

### Persisted `source_raw` serious lane

```bash
./scripts/pg17_scratch_psql.sh \
  --socket-dir /home/peter/.pgrx \
  --sql "ALTER INDEX tqhnsw_real_50k_tq_srccompare_src_m16_idx SET (rerank_source_column = source_raw);"

./scripts/bench_sql_latency_verified_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_tq_srccompare \
  --corpus-table tqhnsw_real_50k_tq_srccompare_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_tq_srccompare_src_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-persisted-source-source_raw-current.summary
```

Repeated at `--query-limit 200` into:

- `tmp/task16-turboquant-persisted-source-source_raw-current-q200.summary`

### Rerank micro-profile

Captured the one-query grouped rerank helper on both modes into:

- `tmp/task16-turboquant-persisted-source-source-current-rerank-profile.csv`
- `tmp/task16-turboquant-persisted-source-source_raw-current-rerank-profile.csv`

### Recall summary

With the persisted `source_raw` reloption still active:

```bash
./scripts/run_real_corpus_recall_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  summary \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_queries \
  --index tqhnsw_real_50k_tq_srccompare_src_m16_idx \
  --corpus-table tqhnsw_real_50k_tq_srccompare_corpus
```

## Artifacts

- `tmp/task16-turboquant-persisted-source-source-current.summary`
- `tmp/task16-turboquant-persisted-source-source_raw-current.summary`
- `tmp/task16-turboquant-persisted-source-source-current-q200.summary`
- `tmp/task16-turboquant-persisted-source-source_raw-current-q200.summary`
- `tmp/task16-turboquant-persisted-source-source-current-rerank-profile.csv`
- `tmp/task16-turboquant-persisted-source-source_raw-current-rerank-profile.csv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260419T005004Z_summary_tqhnsw_real_50k_tq_srccompare_src_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`

## Results

### 1. The supported persisted-source path is real, but the top-line win is modest

On current head, same table, same index, same serious lane:

50-query run:

- `source` mean: `4.746ms`
- `source_raw` mean: `4.746ms`

That run was too noisy to show a small win.

200-query rerun:

- `source` mean: `5.057ms`
- `source_raw` mean: `4.838ms`
- delta: `-0.219ms` / `-4.33%`

So the supported `rerank_source_column = source_raw` path does help the
serious lane on current head, but the win is smaller than the earlier packet
`430` same-table experiment.

### 2. The micro-profile confirms the win is still in heap fetch/decode

One-query grouped rerank helper on the same index:

| metric | `source` | `source_raw` | delta |
|---|---:|---:|---:|
| heap rerank elapsed | `1688us` | `1568us` | `-120us` |
| heap rows fetched | `40` | `40` | `0` |
| heap fetch elapsed | `107us` | `98us` | `-9us` |
| heap decode elapsed | `1488us` | `1386us` | `-102us` |
| heap dot elapsed | `40us` | `40us` | `0us` |

Inference:

- the persisted-source path is active
- the savings are again in heap fetch/decode
- dot-product cost is unchanged
- on this surface the rerank savings are only a few percent of end-to-end scan
  time, so a short 50-query run masked them

### 3. Serious-lane correctness stayed pinned

Recall summary with persisted `source_raw` active:

- `graph_recall_at_10 = 0.9629`
- `ndcg_at_10 = 0.9766229`
- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

That preserves the same recall-preserving serious lane established earlier in
task 16.

## Readout

### 1. Packet `439`'s supported-path productization does carry a measurable runtime win

The result is not just DDL plumbing. On current head, using the persisted
TurboQuant reloption instead of an env override recovers about `4.3%` on the
serious lane when measured with enough queries to overcome noise.

### 2. The remaining task-16 blocker is unchanged

Even with persisted `source_raw`, the serious lane still sits around
`4.8ms-5.1ms`, well above the quantized lane. The measurements still point to
heap rerank/source handling as the limiter, not TurboQuant's traversal kernel
by itself.
