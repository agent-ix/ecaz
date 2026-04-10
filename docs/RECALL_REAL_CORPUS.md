# Real-Corpus Recall Dataset Contract

This document specifies the local dataset contract used by the
`tqhnsw_graph_scan_recall_external_*` probes for running the A4 recall gate on a
real `1536`-dimensional embedding corpus consistent with `NFR-003`.

It is the answer to the contradiction recorded in
`review/217-a4-synthetic-vs-nfr-dataset-gap`: the in-repo synthetic generators
are not a credible gate surface, and `NFR-003` requires DBpedia OpenAI embeddings
or a documented equivalent.

## Chosen Dataset

**Primary:** OpenAI `text-embedding-ada-002` embeddings of DBpedia entity
descriptions, as published by Qdrant under the "DBpedia Entities OpenAI
Embeddings 1M" release.

| Field | Value |
| --- | --- |
| Source | Qdrant `dbpedia-entities-openai-1M` (Hugging Face) |
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

1. Stage the corpus and query files at the documented paths (see
   `scripts/load_real_corpus.py --help`).
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
       --corpus-file /path/to/dbpedia_50k_corpus.tsv \
       --queries-file /path/to/dbpedia_1k_queries.tsv \
       --m 8 16
   ```
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

## Reporting

Real-corpus A4 results MUST be recorded in the same durable style as the
synthetic gate: dataset name, row count, query count, seed, m, ef_search,
graph Recall@10, exact-quantized Recall@10, and a clear pass/fail line against
the published `NFR-003` gates.

The first real-corpus run is documented in
`review/218-a4-real-corpus-recall-lane`.

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
