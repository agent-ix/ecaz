# Review Request: C1 Task16 Ecvector Surface Head-to-Head Measurement

Current head at execution: `b704d7a`

## Context

Packet `441` found the decisive serious-lane lever:

- keep the raw rerank source inline
- stop paying the heap decode / detoast tax

But that packet still used the old duplicate-column research seam:

- `embedding tqvector`
- `source real[]`
- `source_raw bytea`

Packets `442` through `445` changed the product model:

- `ecvector` is now the canonical row type
- `tqvector` is only the narrow TurboQuant artifact/debugging type

Task 16 still needed the real measurement answer on that corrected surface:

1. what does the serious lane look like on default-storage `ecvector`?
2. does the inline-storage win carry over to `ecvector` itself?
3. on the same inline `ecvector` surface, where does TurboQuant now sit
   against PqFastScan?

This packet answers those three questions.

## Important setup note

The existing scratch `postgres` database was stale for this purpose: it still
had the old extension catalog with `tqvector` but no `ecvector`.

So the authoritative run in this packet uses a fresh current-head scratch DB:

```bash
./scripts/install_adr030_pg17_pg_test.sh --pgrx-home /home/peter/.pgrx

./scripts/restart_adr030_scratch.sh \
  --pgrx-home /home/peter/.pgrx \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32
```

Then:

```sql
DROP DATABASE IF EXISTS task16_ecvector WITH (FORCE);
CREATE DATABASE task16_ecvector;
```

and inside that fresh DB:

```sql
CREATE EXTENSION tqvector;
```

Loaded the canonical 50k fixture into `task16_ecvector` from the staged TSVs:

- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv`

using:

```bash
./scripts/load_real_corpus_scratch.sh \
  --db task16_ecvector \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  --m 16
```

## Measurement surfaces

### 1. Default-storage `ecvector`

Derived directly from the canonical `source` column:

```sql
CREATE TABLE tqhnsw_real_50k_ecvector_default_corpus AS
SELECT
  id,
  source,
  encode_to_ecvector(source, 4, 42) AS embedding
FROM tqhnsw_real_50k_corpus;

ANALYZE tqhnsw_real_50k_ecvector_default_corpus;
```

Storage readout:

- `attstorage = 'e'`
- heap bytes: `3,833,856`
- table aux bytes: `828,538,880`
- index bytes (per built index): `68,403,200`

This is the "what current head gives you by default" surface.

### 2. Inline-storage `ecvector`

Same data, but force the canonical row column inline:

```sql
CREATE TABLE tqhnsw_real_50k_ecvector_inline_corpus (
  id bigint,
  source real[],
  embedding ecvector
);

ALTER TABLE tqhnsw_real_50k_ecvector_inline_corpus
  ALTER COLUMN embedding SET STORAGE PLAIN;

INSERT INTO tqhnsw_real_50k_ecvector_inline_corpus
SELECT
  id,
  source,
  encode_to_ecvector(source, 4, 42)
FROM tqhnsw_real_50k_corpus;

ANALYZE tqhnsw_real_50k_ecvector_inline_corpus;
```

Storage readout:

- `attstorage = 'p'`
- heap bytes: `409,600,000`
- table aux bytes: `414,416,896`
- index bytes (per built index): `68,403,200`

So the total table footprint stays in the same class, but the heap/TOAST split
moves from:

- default: `3.8MB heap / 828.5MB aux`

to:

- inline: `409.6MB heap / 414.4MB aux`

### 3. Indexes

Built four `m=16`, `ef_construction=128` indexes:

```sql
CREATE INDEX tqhnsw_real_50k_ecvector_default_turboquant_m16_idx
ON tqhnsw_real_50k_ecvector_default_corpus
USING tqhnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, storage_format = 'turboquant');

CREATE INDEX tqhnsw_real_50k_ecvector_default_pq_fastscan_m16_idx
ON tqhnsw_real_50k_ecvector_default_corpus
USING tqhnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, storage_format = 'pq_fastscan');

CREATE INDEX tqhnsw_real_50k_ecvector_inline_turboquant_m16_idx
ON tqhnsw_real_50k_ecvector_inline_corpus
USING tqhnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, storage_format = 'turboquant');

CREATE INDEX tqhnsw_real_50k_ecvector_inline_pq_fastscan_m16_idx
ON tqhnsw_real_50k_ecvector_inline_corpus
USING tqhnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, storage_format = 'pq_fastscan');
```

## Important measurement note

When both format indexes coexist on the same table, the planner may still pick
the TurboQuant index for the PqFastScan cell. The verified wrapper caught this
correctly and aborted the first default-surface PqFastScan attempt.

So the final readout follows the same isolation discipline used in earlier
task-16 packets:

- capture the TurboQuant result while that index exists
- run the recall summary
- drop the competing TurboQuant index
- then measure the PqFastScan cell on the same table

That isolation was required on both the default and inline `ecvector`
surfaces.

## Commands

### Default `ecvector` TurboQuant

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --db task16_ecvector \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_ecvector_default_turboquant \
  --corpus-table tqhnsw_real_50k_ecvector_default_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_ecvector_default_turboquant_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-ecvector-default-turboquant-q200.summary
```

### Default `ecvector` PqFastScan

```sql
DROP INDEX tqhnsw_real_50k_ecvector_default_turboquant_m16_idx;
```

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --db task16_ecvector \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_ecvector_default_pq_fastscan \
  --corpus-table tqhnsw_real_50k_ecvector_default_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_ecvector_default_pq_fastscan_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-ecvector-default-pq-fastscan-q200.summary
```

### Inline `ecvector` TurboQuant

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --db task16_ecvector \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_ecvector_inline_turboquant \
  --corpus-table tqhnsw_real_50k_ecvector_inline_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_ecvector_inline_turboquant_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-ecvector-inline-turboquant-q200.summary
```

Repeated into:

- `tmp/task16-ecvector-inline-turboquant-q200-rerun.summary`

### Inline `ecvector` PqFastScan

```sql
DROP INDEX tqhnsw_real_50k_ecvector_inline_turboquant_m16_idx;
```

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --db task16_ecvector \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_ecvector_inline_pq_fastscan \
  --corpus-table tqhnsw_real_50k_ecvector_inline_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_ecvector_inline_pq_fastscan_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-ecvector-inline-pq-fastscan-q200.summary
```

Repeated into:

- `tmp/task16-ecvector-inline-pq-fastscan-q200-rerun.summary`

### Recall summaries

Captured on all four measured indexes into:

- `tmp/real_corpus_runs/20260419T150024Z_summary_tqhnsw_real_50k_ecvector_default_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`
- `tmp/real_corpus_runs/20260419T150518Z_summary_tqhnsw_real_50k_ecvector_default_pq_fastscan_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`
- `tmp/real_corpus_runs/20260419T150259Z_summary_tqhnsw_real_50k_ecvector_inline_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`
- `tmp/real_corpus_runs/20260419T150518Z_summary_tqhnsw_real_50k_ecvector_inline_pq_fastscan_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`

## Results

### 1. Default-storage `ecvector` does not capture the packet-`441` win by itself

Default serious lane on the productized row type:

- TurboQuant mean: `5.248ms`
- PqFastScan mean: `5.904ms`

Recall summaries:

- TurboQuant:
  - `graph_recall_at_10 = 0.9629`
  - `ndcg_at_10 = 0.9766229`
  - `mean_abs_score_error = 0`
  - `graph_below_exact_queries = 0`
- PqFastScan:
  - `graph_recall_at_10 = 0.9635`
  - `ndcg_at_10 = 0.976963`
  - `mean_abs_score_error = 0`
  - `graph_below_exact_queries = 0`

Readout:

- current-head `ecvector` is the correct row model
- but with default storage it is still a toast-heavy serious lane
- the packet-`441` breakthrough does **not** appear automatically just by
  switching the row type from `bytea`/duplicate-column plumbing to `ecvector`

### 2. Inline-storage `ecvector` carries the serious-lane win onto the real type

Inline TurboQuant q200:

- first run: `3.427ms`
- confirming rerun: `3.195ms`

Versus default-storage TurboQuant (`5.248ms`):

- confirming delta: `-2.053ms / -39.12%`

Recall stayed pinned:

- `graph_recall_at_10 = 0.9629`
- `ndcg_at_10 = 0.9766229`
- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`

So the packet-`441` finding survives the type migration:

- the lever is still heap storage layout
- not `bytea` specifically
- `ecvector` can realize the same serious-lane shape once kept inline

### 3. On the same inline `ecvector` surface, PqFastScan still leads TurboQuant

Inline PqFastScan q200:

- first run: `2.987ms`
- confirming rerun: `2.954ms`

Against inline TurboQuant confirming rerun (`3.195ms`):

- delta: `-0.241ms / -7.54%`

Recall summaries on the same inline `ecvector` surface:

- TurboQuant:
  - `graph_recall_at_10 = 0.9629`
  - `ndcg_at_10 = 0.9766229`
- PqFastScan:
  - `graph_recall_at_10 = 0.9635`
  - `ndcg_at_10 = 0.976963`

So the task-16 answer is now explicit on the actual row type:

- storage layout is the decisive serious-lane lever
- inline `ecvector` makes TurboQuant credible again
- but on the same inline `ecvector` surface, PqFastScan is still faster and
  slightly higher recall at this `m=16`, `ef_search=128` operating point

## Artifacts

- `tmp/task16-ecvector-default-turboquant-q200.summary`
- `tmp/task16-ecvector-default-pq-fastscan-q200.summary`
- `tmp/task16-ecvector-inline-turboquant-q200.summary`
- `tmp/task16-ecvector-inline-turboquant-q200-rerun.summary`
- `tmp/task16-ecvector-inline-pq-fastscan-q200.summary`
- `tmp/task16-ecvector-inline-pq-fastscan-q200-rerun.summary`
- `tmp/real_corpus_runs/20260419T150024Z_summary_tqhnsw_real_50k_ecvector_default_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`
- `tmp/real_corpus_runs/20260419T150518Z_summary_tqhnsw_real_50k_ecvector_default_pq_fastscan_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`
- `tmp/real_corpus_runs/20260419T150259Z_summary_tqhnsw_real_50k_ecvector_inline_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`
- `tmp/real_corpus_runs/20260419T150518Z_summary_tqhnsw_real_50k_ecvector_inline_pq_fastscan_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`

## Review focus

1. Is the packet drawing the right distinction between:
   - default-storage `ecvector` as the current product default
   - inline-storage `ecvector` as the shape that actually recovers the packet-`441` win?
2. Is the planner-isolation note adequate, or should a future bench helper grow
   an index-disable knob so same-table multi-index format comparisons are less
   awkward?
3. Does this packet justify closing the task-16 checklist items for:
   - `ecvector` head-to-head measurement
   - same-inline-surface TurboQuant vs PqFastScan measurement?
