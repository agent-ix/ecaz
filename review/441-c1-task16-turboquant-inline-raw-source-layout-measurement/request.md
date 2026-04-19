# Review Request: C1 Task16 TurboQuant Inline Raw-Source Layout Measurement

Current head at execution: `4e84c09`

## Context

Packet `440` established the current supported TurboQuant serious lane on
current head:

- same-index `source_raw` beats `source` by `-0.219ms / -4.33%`
- top-line latency still sits at `4.838ms`
- the rerank micro-profile still points at heap source decode

This packet tests the next concrete hypothesis from that readout:

- the remaining serious-lane cost is mostly Postgres heap/TOAST layout
- not the `source_raw` bytea scorer logic itself

So instead of changing any Rust scan kernel, this packet changes only heap
column storage shape and measures what happens.

## Measurement surfaces

All surfaces reuse the same canonical 50k corpus and the same serious-lane
runtime:

```bash
./scripts/restart_adr030_scratch.sh \
  --pgrx-home /home/peter/.pgrx \
  --window 64 \
  --grouped-score-mode binary \
  --rerank-mode heap_f32
```

### Baseline from packet `440`

Current supported surface:

- table: `tqhnsw_real_50k_tq_srccompare_corpus`
- index: `tqhnsw_real_50k_tq_srccompare_src_m16_idx`
- reloptions:
  - `build_source_column = source`
  - `rerank_source_column = source_raw`
  - `storage_format = turboquant`

Current-head q200 latency:

- `4.838ms`

Current-head rerank micro-profile:

- heap rerank total `1568us`
- heap fetch `98us`
- heap decode `1386us`
- heap dot `40us`

### Control 1: raw-only default TOAST layout

Built a source-less raw-only corpus:

```sql
CREATE TABLE tqhnsw_real_50k_tq_rawonly_corpus AS
SELECT id, source_raw, embedding
FROM tqhnsw_real_50k_tq_srccompare_corpus;

ANALYZE tqhnsw_real_50k_tq_rawonly_corpus;
```

Storage readout:

- avg row size: `6967` bytes
- heap: `43MB`
- toast: `394MB`

Built:

```sql
CREATE INDEX tqhnsw_real_50k_tq_rawonly_m16_idx
ON tqhnsw_real_50k_tq_rawonly_corpus
USING tqhnsw (embedding)
WITH (
  m = 16,
  ef_construction = 128,
  rerank_source_column = source_raw,
  storage_format = turboquant
);
```

This control asks:

- is simply removing the sibling `source real[]` enough?

### Candidate: mixed inline raw-source layout

Built a corpus that keeps `source` for correctness tooling but forces the
runtime rerank column in-page:

```sql
CREATE TABLE tqhnsw_real_50k_tq_mixed_inline_corpus (
  id bigint,
  source real[],
  source_raw bytea,
  embedding tqvector
);

ALTER TABLE tqhnsw_real_50k_tq_mixed_inline_corpus
  ALTER COLUMN source SET STORAGE EXTERNAL;

ALTER TABLE tqhnsw_real_50k_tq_mixed_inline_corpus
  ALTER COLUMN source_raw SET STORAGE PLAIN;

INSERT INTO tqhnsw_real_50k_tq_mixed_inline_corpus
SELECT id, source, source_raw, embedding
FROM tqhnsw_real_50k_tq_srccompare_corpus;

ANALYZE tqhnsw_real_50k_tq_mixed_inline_corpus;
```

Storage readout:

- avg row size: `13135` bytes
- heap: `390MB`
- toast: `396MB`

Compared with the supported packet-`440` surface:

- same total footprint class (`786MB` vs `833MB` overall)
- but the heap/TOAST split moves from `43MB / 790MB` to `390MB / 396MB`
- `source_raw` is inline (`STORAGE PLAIN`)
- `source` is pushed out of the hot path (`STORAGE EXTERNAL`)

Built:

```sql
CREATE INDEX tqhnsw_real_50k_tq_mixed_inline_m16_idx
ON tqhnsw_real_50k_tq_mixed_inline_corpus
USING tqhnsw (embedding)
WITH (
  m = 16,
  ef_construction = 128,
  build_source_column = source,
  rerank_source_column = source_raw,
  storage_format = turboquant
);
```

## Commands

### Raw-only control

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_tq_rawonly \
  --corpus-table tqhnsw_real_50k_tq_rawonly_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_tq_rawonly_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-rawonly-source_raw-q200.summary
```

### Mixed-inline candidate

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_tq_mixed_inline \
  --corpus-table tqhnsw_real_50k_tq_mixed_inline_corpus \
  --query-table tqhnsw_real_50k_queries \
  --index-name tqhnsw_real_50k_tq_mixed_inline_m16_idx \
  --m 16 \
  --ef-search 128 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-turboquant-mixed-inline-source_raw-q200.summary
```

Captured the grouped rerank micro-profile into:

- `tmp/task16-turboquant-rawonly-source_raw-rerank-profile.csv`
- `tmp/task16-turboquant-mixed-inline-source_raw-rerank-profile.csv`

### Mixed-inline recall summary

```bash
./scripts/run_real_corpus_recall_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  summary \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_queries \
  --index tqhnsw_real_50k_tq_mixed_inline_m16_idx \
  --corpus-table tqhnsw_real_50k_tq_mixed_inline_corpus
```

## Artifacts

- `tmp/task16-turboquant-persisted-source-source_raw-current-q200.summary`
- `tmp/task16-turboquant-rawonly-source_raw-q200.summary`
- `tmp/task16-turboquant-rawonly-source_raw-rerank-profile.csv`
- `tmp/task16-turboquant-mixed-inline-source_raw-q200.summary`
- `tmp/task16-turboquant-mixed-inline-source_raw-rerank-profile.csv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260419T021226Z_summary_tqhnsw_real_50k_tq_mixed_inline_m16_idx_m16_ef128_tqhnsw_real_50k_queries.tsv`

## Results

### 1. Raw-only without inline storage is a dead end

Raw-only default-TOAST surface:

- q200 mean: `5.104ms`
- delta vs packet `440` supported `source_raw` surface (`4.838ms`):
  `+0.266ms / +5.50%`

Rerank micro-profile:

- heap rerank total: `1822us`
- heap fetch: `105us`
- heap decode: `1636us`
- heap dot: `41us`

So simply deleting the sibling `source` column is **not** enough. The default
raw-only layout is still toast-heavy and actually regresses the serious lane.

### 2. Inline `source_raw` is the first post-`440` lever that really moves the serious lane

Mixed-inline surface:

- q200 mean: `3.137ms`
- delta vs packet `440` supported `source_raw` surface (`4.838ms`):
  `-1.701ms / -35.16%`
- delta vs the raw-only default-TOAST control (`5.104ms`):
  `-1.967ms / -38.54%`

Rerank micro-profile:

| metric | packet `440` supported `source_raw` | mixed-inline | delta |
|---|---:|---:|---:|
| heap rerank total | `1568us` | `216us` | `-1352us` |
| heap fetch | `98us` | `134us` | `+36us` |
| heap decode | `1386us` | `1us` | `-1385us` |
| heap dot | `40us` | `50us` | `+10us` |

That is the cleanest serious-lane readout in task 16 so far:

- fetch does not get better
- dot product does not get better
- decode essentially disappears
- the top-line query latency follows the rerank bucket down

### 3. Mixed-inline preserves the serious-lane recall point

Recall summary on `tqhnsw_real_50k_tq_mixed_inline_m16_idx`:

- `graph_recall_at_10 = 0.9629`
- `ndcg_at_10 = 0.9766229`
- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

That is identical to the recall-preserving serious lane already established in
packets `429`, `430`, and `440`.

## Readout

### 1. The remaining serious-lane problem was mostly heap-source layout, not scorer math

Packet `437` already showed that scorer work (lever 4/5) barely moved the
`heap_f32` lane. Packet `440` showed that switching the source representation
to packed `bytea` helped a bit. This packet closes the loop:

- the big remaining win comes from keeping `source_raw` inline
- not from another exact-score kernel change

### 2. The next justified implementation is a first-class inline raw-f32 source path

The strong result is **not** "raw-only is faster". Raw-only default TOAST
layout lost.

The strong result is:

- `source_raw` inline
- `source` kept available but pushed external
- same recall
- serious lane down to `3.137ms`

That points toward product work such as:

- a first-class raw-f32 heap type / column path
- or documented / automated storage-policy support for inline rerank sources

Either way, the next task-16 implementation should target heap storage layout,
not another scorer variant.
