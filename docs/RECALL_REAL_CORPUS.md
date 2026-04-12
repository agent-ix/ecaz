# Real-Corpus Recall Dataset Contract

This document specifies the local dataset contract used by the
`tqhnsw_graph_scan_recall_external_*` probes for running the A4 recall gate on a
real `1536`-dimensional embedding corpus consistent with `NFR-003`.

It is the answer to the contradiction recorded in
`review/217-a4-synthetic-vs-nfr-dataset-gap`: the in-repo synthetic generators
are not a credible gate surface, and `NFR-003` requires DBpedia OpenAI embeddings
or a documented equivalent.

## Chosen Dataset

**Primary:** OpenAI `text-embedding-3-large` `1536`-dimensional embeddings of
DBpedia entity descriptions, as published by Qdrant on Hugging Face.

| Field | Value |
| --- | --- |
| Source | Qdrant `dbpedia-entities-openai3-text-embedding-3-large-1536-1M` (Hugging Face) |
| License | OpenAI embeddings + DBpedia text are public; redistributable per the published terms. Confirm at fetch time. |
| Total rows | 1,000,000 |
| Dimensionality | 1536 |
| Element type | `float32` (compatible with `real[]` in Postgres) |
| Distance | inner product / cosine on unit-normalized vectors |

The probe path is dataset-agnostic. The local file format is the contract — any
real `1536`-dim corpus that lands in the format below will work.

### Default Working Subset

A1 / A4 measurement does not require the full 1M rows. The default working
subset for the recall gate is:

| Subset | Rows | Queries | Notes |
| --- | --- | --- | --- |
| `tqhnsw_real_50k` | 50,000 corpus | 1,000 queries | matches `NFR-003` headline shape |
| `tqhnsw_real_10k` | 10,000 corpus | 200 queries | optional smaller surface for fast iteration |

Larger subsets (full 1M) are allowed but should be opt-in: A4 needs the 50k
shape because that is the surface `NFR-003` declares its targets against.

### Canonical Selection Rule

The default subsets are **not** "any random 50k rows". They are pinned by a
deterministic rule over the full parquet release:

- sort the full dataset by the source parquet id column ascending
- the current Hugging Face parquet uses `_id`, so the canonical sort key is
  `_id` ascending lexicographic
- `tqhnsw_real_50k` corpus: rows `[0, 49_999]`
- `tqhnsw_real_50k` queries: rows `[50_000, 50_999]`
- `tqhnsw_real_10k` corpus: rows `[0, 9_999]`
- `tqhnsw_real_10k` queries: rows `[10_000, 10_199]`

The emitted TSV ids are deterministic global row indices from that sorted
ordering. The original source ids remain in the manifest as
`first_source_id` / `last_source_id`.

The canonical conversion recipe is implemented in
`scripts/qdrant_dbpedia_to_tsv.py`. Official first-run A4 numbers should be
derived from that script, not from ad hoc parquet slicing.

## Local File Contract

The loader and probe path do not download anything. The dataset is staged
locally as two simple files. Format choices favor auditability and trivial
inspection over compactness:

### Corpus file: `<basename>_corpus.tsv`

- Tab-separated values
- One row per vector
- Two columns:
  1. `id` — `int8`, unique within the file
  2. `embedding_json` — a JSON array of `1536` floats, e.g. `[0.0123,-0.456,...]`
- No header row
- UTF-8, Unix line endings

### Query file: `<basename>_queries.tsv`

- Same shape as the corpus file
- Distinct `id` space (queries are not constrained to be a subset of the corpus)

### Why TSV + JSON arrays

- Two columns means trivial parsing: `split('\t', 1)` then JSON-decode the second
  column. There is no comma escaping inside the JSON because the row is
  tab-separated.
- JSON arrays survive copy/paste and `head -1`, unlike binary formats.
- We do not optimize for I/O throughput on the loader path. The reusable
  fixture flow loads once.

### Manifest File: `<basename>_manifest.json`

Canonical conversions also emit a sibling manifest:

- `manifest_version`
- `prefix`
- dataset/source metadata (see below)
- the exact selection rule
- dimensionality
- per-file row counts
- per-file SHA-256 digests
- first/last ids for the corpus and query files

The dataset/source metadata fields, expanded, are:

- `source_dataset`: human-readable dataset label (e.g. `"Qdrant
  dbpedia-entities-openai3-text-embedding-3-large-1536-1M"`), not a path
- `source_parquet`: the local absolute path used at conversion time, kept
  as a debugging hint only — **the loader does not verify this field** and
  reviewers on a different machine are expected to ignore it
- `source_parquet_basename`: portable basename (just the file or directory
  name) of `source_parquet`, verified by the loader
- `source_parquet_shard_basenames`: sorted list of per-shard parquet file
  basenames that were actually iterated, verified by the loader

The portable basename fields are what reviewers cross-check; the absolute
`source_parquet` path exists only for the developer who ran the conversion.
The loader accepts manifests that omit the portable fields (older
manifests) but rejects any present-but-non-portable values (absolute
paths, non-strings).

The loader auto-discovers and verifies this manifest when the staged files
follow the canonical `<basename>_{corpus,queries}.tsv` naming pattern. If the
hashes differ, the loader aborts unless the operator passes
`--allow-manifest-mismatch`.

## Reusable Fixture Flow

The probe path is built around one durable expectation:

1. Load the corpus and query files **once** into Postgres.
2. Build the tqhnsw index for each `m` **once**.
3. Rerun the probes any number of times against the existing tables and
   indexes, without rebuilding the corpus.

The loader script is idempotent. If the corpus table already has the expected
row count and the index already exists with the expected `(m, ef_construction)`,
the loader skips the corresponding step and logs `skipped: already present`.

This is the same one-time-load / repeated-rerun discipline used by the synthetic
fixture-backed gate (`tqhnsw_graph_scan_recall_fixture_gate_*`).

The loader also logs mean/min/max L2 norm statistics for the staged corpus and
query files. If the vectors are not close to unit norm, it emits a warning so
the operator can catch a metric-contract mismatch before recording recall
numbers.

## Schema

The loader creates these tables in the target database:

```sql
CREATE TABLE <prefix>_corpus (
    id          bigint PRIMARY KEY,
    source      real[] NOT NULL,
    embedding   tqvector
);

CREATE TABLE <prefix>_queries (
    id          bigint PRIMARY KEY,
    source      real[] NOT NULL
);
```

The `<prefix>_corpus.embedding` column is populated as
`encode_to_tqvector(source, 4, 42)` so it lives next to the source for
`source`-build-mode tqhnsw indexing and exact-quantized comparison probes.

The tqhnsw indexes are created via the existing `build_source_column = 'source'`
path so the graph is built on raw `source` vectors rather than re-decoded
quantized values:

```sql
CREATE INDEX <prefix>_m8_idx ON <prefix>_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (m = 8, ef_construction = 128, build_source_column = 'source');

CREATE INDEX <prefix>_m16_idx ON <prefix>_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (m = 16, ef_construction = 128, build_source_column = 'source');
```

## How to Use

1. Convert the parquet release into canonical TSV + manifest files:
   ```bash
   python3 scripts/qdrant_dbpedia_to_tsv.py \
       --profile tqhnsw_real_50k \
       --parquet /path/to/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data \
       --output-dir /path/to/staged
   ```
   This requires `pyarrow` to be installed locally. The converter writes:
   - `tqhnsw_real_50k_corpus.tsv`
   - `tqhnsw_real_50k_queries.tsv`
   - `tqhnsw_real_50k_manifest.json`
   If you are targeting the repo-local scratch `pg17` cluster, the same flow is
   available as a single command:
   ```bash
   scripts/prepare_real_corpus_scratch.sh \
       --profile tqhnsw_real_50k \
       --parquet /path/to/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data \
       --output-dir /path/to/staged \
       --m 8 --m 16
   ```
   That wrapper runs the canonical converter first, then calls
   `scripts/load_real_corpus_scratch.sh` on the emitted TSVs.
2. Install the `pg_test` build of the extension before running the SQL
   recall surfaces from `psql`:
   ```bash
   PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx install --release --test \
       --pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config \
       --features 'pg17 pg_test' --no-default-features
   ```
   If `psql` is not already on `PATH`, set `TQV_PSQL_BIN` to an explicit
   client binary before invoking the loader.
3. Run the loader:
   ```bash
   PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \
       --prefix tqhnsw_real_50k \
       --corpus-file /path/to/staged/tqhnsw_real_50k_corpus.tsv \
       --queries-file /path/to/staged/tqhnsw_real_50k_queries.tsv \
       --m 8 16
   ```
   If a sibling `tqhnsw_real_50k_manifest.json` exists, the loader verifies it
   automatically before loading. You can also pass `--manifest-file` explicitly.
   For the repo-local scratch `pg17` cluster, use
   `scripts/load_real_corpus_scratch.sh` to pin the expected socket, port,
   database, and `psql` binary. The scratch helper defaults `PGDATABASE` to
   `postgres` because the pgrx scratch cluster does not provision a separate
   `tqvector_bench` database.
4. Run the A4 gate report:
   ```sql
   SELECT * FROM tqhnsw_graph_scan_recall_external_gate_report(
       'tqhnsw_real_50k_corpus',
       'tqhnsw_real_50k_queries',
       'tqhnsw_real_50k'
   );
   ```
   This emits one row per A4 configuration:
   ```
   m | ef_search | recall_at_10 | gate_recall_at_10 | passes_gate
   8 |        40 |        0.xx  |             null  |  true
   8 |       128 |        0.xx  |              0.89 | true|false
   8 |       200 |        0.xx  |              0.93 | true|false
   16|       200 |        0.xx  |              0.97 | true|false
   ```
5. For per-query detail, use:
   ```sql
   SELECT * FROM tqhnsw_graph_scan_recall_external_summary(
       'tqhnsw_real_50k_corpus',
       'tqhnsw_real_50k_queries',
       'tqhnsw_real_50k_m8_idx',
       8,
       128
   );
   ```

## Diagnostics

When the gate report from step 4 lands below target, three additional surfaces
turn the yes/no result into a diagnosable result. All three reuse the canonical
`<prefix>_corpus` / `<prefix>_queries` tables and the same per-`m` indexes built
in step 1 — they do not load anything new and they build the external recall
context exactly once per call. Each is gated behind the `pg_test` build of the
extension, the same as the existing `tqhnsw_graph_scan_recall_external_*`
surfaces.

### Per-query recall histogram

```sql
SELECT * FROM tqhnsw_graph_scan_recall_histogram(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k_m8_idx',
    8,
    128
);
```

Returns 11 rows, one per top-10 recall bucket `0..=10`:

```
 recall_bucket | query_count | query_fraction
---------------+-------------+----------------
             0 |           0 |        0.00000
             1 |           0 |        0.00000
             ...
             9 |          14 |        0.07000
            10 |         186 |        0.93000
```

A healthy run concentrates almost everything in buckets 9 and 10. A run that
spreads across mid buckets means recall is mediocre everywhere; a run that
concentrates a small tail in low buckets means a few queries are catastrophic
and the rest are fine. The two failure modes have completely different fixes.

### `ef_search` sweep on a single fixture

```sql
SELECT * FROM tqhnsw_graph_scan_recall_ef_sweep(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k_m8_idx',
    8,
    ARRAY[40, 64, 100, 128, 160, 200, 300, 500]
);
```

Returns one row per `ef_search` value:

```
 m | ef_search | recall_at_10 | exact_quantized_recall_at_10 | mean_abs_score_error | mean_query_latency_ms
---+-----------+--------------+------------------------------+----------------------+-----------------------
 8 |        40 |        0.xx  |                        0.xx  |               0.xxxx |                  x.xx
 8 |        64 |        0.xx  |                        0.xx  |               0.xxxx |                  x.xx
 ...
```

The recall context is built once and reused for every `ef_search`, so the
sweep is the cheapest way to walk the recall/latency Pareto frontier on a
fixed fixture. `mean_query_latency_ms` is the wall-clock spent inside the
graph scan loop divided by the query count — it includes a small constant
NDCG/MAE/Spearman bookkeeping cost but is dominated by the graph traversal.

### Exact-vs-approximate diff for failing queries

```sql
SELECT * FROM tqhnsw_graph_scan_recall_failure_breakdown(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k_m8_idx',
    8,
    128,
    8  -- list every query whose top-10 recall is < 8 (i.e. missed >= 3 of 10)
);
```

Returns one row per query whose graph top-10 recall is strictly less than the
threshold:

```
 query_index | graph_recall_at_10 | exact_quantized_recall_at_10 | missed_ids
-------------+--------------------+------------------------------+-----------------
          17 |                  6 |                          10  | {1234,5678,...}
         142 |                  4 |                           4  | {314,159,...}
```

This is the single diagnostic that distinguishes "the graph is the
bottleneck" from "the quantizer is the bottleneck". If a query misses several
items for the graph but zero for exact-quantized, the graph is the problem and
raising `ef_search` will help. If both surfaces miss the same items, the
quantizer is the problem and no amount of `ef_search` will fix it. The
`missed_ids` column lists corpus ids that neither the graph top-10 nor the
exact-quantized top-10 found, which is the smallest reproducible target for a
hand inspection.

## Reporting

Real-corpus A4 results MUST be recorded in the same durable style as the
synthetic gate: dataset name, row count, query count, seed, m, ef_search,
graph Recall@10, exact-quantized Recall@10, and a clear pass/fail line against
the published `NFR-003` gates.

For the first official DBpedia run, commit the generated manifest alongside the
benchmark/report packet so other reviewers can verify they are measuring the
same staged subset.

The first real-corpus run is documented in
`review/218-a4-real-corpus-recall-lane`.

## Reusing the Loaded Tables for NFR-001 Latency

The same `<prefix>_corpus`, `<prefix>_queries`, and `<prefix>_m{N}_idx`
artifacts produced by `scripts/load_real_corpus.py` for the A4 recall lane
also serve the `NFR-001` query-latency lane: load and bench are decoupled,
so once the loader has built the tables and indexes there is no second
load step. The raw reporting surface is `scripts/bench_sql_latency.sh`, but
durable `NFR-001` artifacts should go through the planner-verified launcher
`scripts/bench_sql_latency_verified.sh`, which first checks a representative
`EXPLAIN` plan and refuses to run unless the planner selects the expected
`<prefix>_m{N}_idx`. The verified launcher currently accepts one effective
`m` per invocation so the chosen index is unambiguous. A worked example
against the already-loaded `tqhnsw_real_10k` fixture:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8.summary > /tmp/nfr1_real_10k_m8.stdout
```

The wrapper pins the same socket / port / database / `psql` binary as
`load_real_corpus_scratch.sh`, so the "load then bench" path against the
repo-local pgrx scratch cluster needs no per-run env setup. Each
`(m, ef_search)` cell emits one summary line with `p50`, `p95`, `p99`,
`mean`, `min`, `max`, `server_qps`, and total cell wall time against the
same query set used by the recall probes. `server_qps` is derived from the
summed per-query `EXPLAIN (ANALYZE)` execution times, so it reflects server
execution rather than `psql` process-spawn overhead. Stdout also emits a
host / GUC banner (`CPU`, `RAM`, `shared_buffers`, `work_mem`,
`max_parallel_workers_per_gather`, and the operator-supplied `--cache-state`
label), which is why the canonical command redirects stdout to a companion
artifact file. See `spec/non-functional/NFR-001-query-latency.md` for the
gate target.

If the planner surface is not active for the target index, the verified
launcher aborts before timing and prints the representative plan. That is the
guard against silently recording a sequential `Sort -> Seq Scan` plan as an
HNSW artifact. On current `main`, the canonical `tqhnsw_real_10k` `m=8`
surface is planner-visible and produces durable NFR-001 artifacts through the
verified launcher. When comparing alternate `m` values on the same loaded
corpus, the planner may naturally prefer the cheaper sibling index; in that
case, use an isolated prefix (for example `tqhnsw_real_10k_m16only`) so the
verified run measures the intended index honestly rather than forcing the
planner to lie.

## Troubleshooting

### Scratch DB missing the new `tests.tqhnsw_graph_scan_recall_external_*` functions

If the scratch cluster already has `tqvector` installed from an older
same-version `pg_test` build, rerunning `cargo pgrx install` updates the SQL on
disk but does not refresh the already-created SQL objects in the database.

For the scratch cluster, the fix is:

```sql
DROP EXTENSION IF EXISTS tqvector CASCADE;
CREATE EXTENSION tqvector CASCADE;
```

This will drop dependent scratch relations. Use a clean scratch database or be
prepared to reload the fixture afterward.

## What This Document Does Not Cover

- Bulk dataset acquisition. The user is responsible for obtaining the corpus.
  The repo never checks in dataset binaries.
- Latency benchmarking. `NFR-001` work uses the same loader path but a
  different reporting surface (`scripts/bench_sql_latency.sh`).
- Insert drift / vacuum recall. Both follow `A5` / `A6` and re-use the loaded
  tables once they exist.
- The published external reference anchor. That is a one-time oracle on a
  separate larger profile and is documented in
  `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`. The anchor reuses this loader.
