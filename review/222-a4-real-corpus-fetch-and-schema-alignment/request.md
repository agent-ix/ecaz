# Review Request: A4 Real-Corpus Fetch + Schema Alignment

## Context

Branch:
- `fix/a4-real-corpus-recall-lane`

Prior packets:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/219-a4-real-corpus-loader-smoke/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`
- `review/221-a4-real-corpus-subset-manifest-contract/request.md`

This slice moves the real-corpus lane from "contract is ready" to "the actual
dataset is present locally and the canonical converter works against the real
parquet schema".

It also records the failed converter paths encountered while moving from toy
fixtures to the real Hugging Face/Qdrant parquet.

## What Landed

### 1. The real parquet is now staged locally

Downloaded the Hugging Face dataset into:

- `/home/peter/dev/datasets/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M`

The download contains `26` parquet shards under the `data/` directory.

This is the first time the lane has operated on the actual external DBpedia
corpus instead of synthetic or tiny smoke TSVs.

### 2. The converter now matches the real parquet schema

`scripts/qdrant_dbpedia_to_tsv.py` originally assumed:

- integer `id`
- one of the generic embedding column names already listed in the script
- `pyarrow.dataset(...)` as the parquet access path

The real parquet does not match that assumption. Its schema is:

- `_id`
- `title`
- `text`
- `text-embedding-3-large-1536-embedding`

The converter now handles that cleanly:

- auto-detects `_id` as the id column
- auto-detects `text-embedding-3-large-1536-embedding` as the vector column
- sorts on the real source `_id` lexicographically
- emits deterministic numeric TSV ids as global sorted row indices
- records the original source-id boundary in the manifest via
  `first_source_id` / `last_source_id`

### 3. The converter now uses a lower-level parquet scan path

The earlier `pyarrow.dataset(...)` path was too memory-hungry for the real
1M-row corpus and never reached output emission in a reasonable state.

The landed path uses `pyarrow.parquet.ParquetFile.iter_batches(...)` directly
over the shard list so the converter only reads the `_id` and embedding columns
it actually needs.

This is the main structural change in the code:

- `scripts/qdrant_dbpedia_to_tsv.py`

### 4. Real `10k` and `50k` canonical subsets are staged

The converter successfully wrote:

- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_corpus.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_queries.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json`

and:

- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_manifest.json`

### 5. Real loader path is proven on the actual corpus

The scratch loader successfully loaded the real `10k` subset and built the
`m=8` index:

- tables: `tqhnsw_real_10k_corpus`, `tqhnsw_real_10k_queries`
- index: `tqhnsw_real_10k_m8_idx`
- counts: `10000` corpus rows, `200` query rows

The real `50k` subset also loaded successfully:

- `50000` corpus rows
- `1000` query rows
- unit-norm checks passed on both corpus and queries

At the time of this packet, the remaining in-flight step was the scratch
`CREATE INDEX tqhnsw_real_50k_m8_idx ...` build.

### 6. The one-shot scratch helper now matches the real parquet defaults

`scripts/prepare_real_corpus_scratch.sh` no longer hardcodes the older `id`
assumption. It now:

- lets the converter auto-detect the real parquet id/vector columns by default
- prefers the sibling `../datasets/.venv/bin/python` interpreter when present
- still allows explicit `PYTHON`, `--id-column`, and `--vector-column`
  overrides

That makes the "single command" path consistent with the actual fetched
Qdrant/Hugging Face release instead of only working when the caller already
knew the schema.

## Evidence

### Real dataset fetch

The Hugging Face download completed with all parquet shards present under:

- `/home/peter/dev/datasets/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data`

### Failed path 1: numeric-id assumption was wrong

Initial converter attempt against the real parquet failed with:

```text
ValueError: invalid literal for int() with base 10: '<dbpedia:Parabolic_reflector>'
```

That is the direct evidence that the real id column is string-valued and the
toy-fixture assumption `int(id)` was invalid.

### Failed path 2: generic schema inference was wrong

Initial converter attempt without explicit column names failed with:

```text
ValueError: could not infer vector column from parquet schema ['_id', 'title', 'text', 'text-embedding-3-large-1536-embedding']; pass --vector-column explicitly
```

That is what prompted the current auto-detect update for the real dataset.

### Schema auto-detect now resolves the real columns

A direct smoke against the staged parquet now returns:

```text
_id
text-embedding-3-large-1536-embedding
```

That is the intended no-flags converter path for the current Hugging Face
dataset.

### Failed path 3: high-level dataset scan path was too heavy

The original `pyarrow.dataset(...).to_table(...)` / scanner path stayed stuck
in the id-materialization stage for minutes, with RSS in the multi-gigabyte
range (`~5 GB`) and no output files emitted.

The direct `ParquetFile.iter_batches(...)` path is the one that actually
completed the real `10k` and `50k` conversions.

### Real `10k` converter completion

Observed output:

```text
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_corpus.tsv
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_queries.tsv
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json
[converter] profile=tqhnsw_real_10k corpus_rows=10000 query_rows=200 sort_key='_id ascending lexicographic'
```

### Real `10k` loader completion

Observed loader output included:

```text
[loader] verified manifest /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json for prefix tqhnsw_real_10k
[loader] tqhnsw_real_10k_corpus corpus norms: count=10000 mean=1.000000 min=1.000000 max=1.000000
[loader] tqhnsw_real_10k_queries queries norms: count=200 mean=1.000000 min=1.000000 max=1.000000
[loader] done. corpus=tqhnsw_real_10k_corpus (10000 rows), queries=tqhnsw_real_10k_queries (200 rows), m=[8]
```

And direct scratch-cluster checks confirmed:

```text
tqhnsw_real_10k_corpus r
tqhnsw_real_10k_m8_idx i
tqhnsw_real_10k_queries r
```

with row counts:

```text
10000  200
```

### Real `50k` converter completion

Observed output:

```text
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_manifest.json
[converter] profile=tqhnsw_real_50k corpus_rows=50000 query_rows=1000 sort_key='_id ascending lexicographic'
```

### Real `50k` loader progress

Observed loader output included:

```text
[loader] verified manifest /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_manifest.json for prefix tqhnsw_real_50k
[loader] tqhnsw_real_50k_corpus corpus norms: count=50000 mean=1.000000 min=1.000000 max=1.000000
[loader] tqhnsw_real_50k_queries queries norms: count=1000 mean=1.000000 min=1.000000 max=1.000000
[loader] building tqhnsw_real_50k_m8_idx (m=8, ef_construction=128) ...
```

Separate scratch-cluster checks confirmed the loaded table counts:

```text
50000  1000
```

### One-shot helper smoke against the real parquet

Observed output from:

```bash
scripts/prepare_real_corpus_scratch.sh \
    --profile tqhnsw_real_10k \
    --parquet /home/peter/dev/datasets/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data \
    --output-dir /home/peter/dev/datasets/tqhnsw_real_10k \
    --m 8
```

included:

```text
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_corpus.tsv
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_queries.tsv
[converter] wrote /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json
[converter] profile=tqhnsw_real_10k corpus_rows=10000 query_rows=200 sort_key='_id ascending lexicographic'
[loader] verified manifest /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json for prefix tqhnsw_real_10k
```

The helper then failed when `psql` tried to reach the scratch cluster from
inside the sandbox. That is an environment boundary, not a schema-alignment
failure: the converter defaults and manifest handoff both worked without
explicit `--id-column` / `--vector-column` flags.

## Why This Matters

Before this slice, the real-corpus lane had the right interface but not the
actual external corpus. After this slice:

- the real dataset is present locally
- the converter is proven on the real Hugging Face schema
- canonical subsets and manifests exist on disk
- the loader is proven on the actual corpus

That means the next lane is no longer "finish data acquisition". It is "run the
real recall probes on the actual DBpedia-derived fixture and decide what the
true A4 number is on that corpus".

## Files

- `scripts/qdrant_dbpedia_to_tsv.py`
- `scripts/prepare_real_corpus_scratch.sh`
- `docs/RECALL_REAL_CORPUS.md`
- `plan/tasks/12-real-corpus-recall.md`
