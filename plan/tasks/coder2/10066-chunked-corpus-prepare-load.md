# Task 10066: Chunked Corpus Prepare and Resumable Load

Status: **proposed**

## Context

`ecaz-cli corpus prepare` currently emits one corpus TSV, one query TSV, and one
manifest for a selected parquet subset. For the real DBPedia 990k/10k profile
(`ec_hnsw_real_ann_benchmarks_anchor`), the corpus TSV is expected to be around
20 GB.

The current prepare implementation also materializes selected rows before
writing the output files. For 990k x 1536 `f32`, raw vector storage alone is
about 5.7 GiB, and `HashMap` / ID / `Vec` overhead can push peak memory much
higher. This is acceptable for a large local workstation but weak as a durable
benchmark workflow.

`CREATE INDEX` remains PostgreSQL-atomic and is not resumable. This task is
only about making corpus preparation and table load resumable up to the point
where an index build can be rerun independently.

## Goal

Add a chunked corpus artifact format and loader path so large real-corpus
fixtures can be prepared and loaded with bounded memory and resume at chunk
boundaries after interruption.

## Non-Goals

- Do not make PostgreSQL `CREATE INDEX` resumable.
- Do not change `ec_hnsw` index build semantics.
- Do not remove support for the existing single-TSV fixture format.
- Do not require network access; this operates on already-staged parquet files.

## Proposed Artifact Layout

For prefix `ec_hnsw_real_ann_benchmarks_anchor`, emit:

```text
<output-dir>/
  ec_hnsw_real_ann_benchmarks_anchor_manifest.json
  corpus/
    corpus-00000.tsv
    corpus-00001.tsv
    ...
  queries/
    queries-00000.tsv
    ...
```

The manifest should record:

- source parquet path and shard basenames
- id column and vector column
- profile name, corpus row count, query row count, dimension
- chunk size policy
- for each chunk:
  - relative path
  - logical kind: `corpus` or `queries`
  - start row / end row in the canonical output sequence
  - row count
  - byte length
  - sha256

Write chunks through temporary filenames and atomically rename only after the
chunk checksum and row count are known.

## Prepare Design

Preserve the existing canonical selection rule:

1. Pass 1 scans parquet IDs and determines the sorted-id prefix of
   `corpus_rows + query_rows`.
2. Split the selected sorted IDs into corpus and query ID sets exactly as the
   current profile does.
3. Pass 2 streams parquet batches and writes selected rows to chunk files.

Important implementation point: pass 2 should not retain every selected vector
in memory. It can retain selected ID membership and per-ID output ordinal, then
write rows into deterministic temporary chunk writers as vectors are found.

Because parquet scan order may not equal canonical sorted-ID output order,
choose one of these approaches:

- **Preferred:** write selected rows to per-chunk temporary stores keyed by
  output ordinal, then sort each chunk in memory before final TSV write. Chunk
  memory is bounded by chunk size.
- **Acceptable first pass:** retain the selected ID -> ordinal map and a small
  number of chunk buffers, flushing completed chunks once all rows for that
  chunk have been seen. This must enforce a hard memory bound or fall back to a
  temp-file spill.

Default chunk size should be configurable. Start with a conservative default
such as 10k or 25k rows per corpus chunk for 1536-dim vectors.

## Load Design

Add a chunk-aware load path to `ecaz-cli corpus load`.

The loader should:

1. Read and validate the chunk manifest.
2. Create the corpus/query tables if absent.
3. Maintain a loader state table, for example:

```sql
CREATE TABLE IF NOT EXISTS ecaz_corpus_load_state (
    prefix text NOT NULL,
    chunk_kind text NOT NULL,
    chunk_path text NOT NULL,
    chunk_sha256 text NOT NULL,
    row_count bigint NOT NULL,
    loaded_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (prefix, chunk_kind, chunk_path)
);
```

4. Before loading a chunk, verify its byte length and sha256 match the manifest.
5. Load one chunk per transaction.
6. Insert the state row in the same transaction as the chunk COPY.
7. On resume, skip chunks whose state row matches the manifest.
8. After all chunks load, verify final corpus/query row counts match the
   manifest.

The loader should reject partial/inconsistent state instead of silently
continuing. If a chunk's table rows exist but the state row is missing, require
operator cleanup or provide an explicit repair command.

## CLI Surface

Suggested additions:

```text
ecaz corpus prepare \
  --profile ec_hnsw_real_ann_benchmarks_anchor \
  --parquet /path/to/parquet/data \
  --output-dir /path/to/output \
  --chunk-rows 25000

ecaz corpus load \
  --prefix tqhnsw_real_ann_benchmarks_anchor \
  --manifest-file /path/to/output/ec_hnsw_real_ann_benchmarks_anchor_manifest.json \
  --chunked
```

Compatibility:

- Existing `--corpus-file` / `--queries-file` single-file load must keep
  working.
- Chunked load can be selected by `--manifest-file` plus a manifest shape, or
  by an explicit `--chunked` flag.

## Validation

Unit tests:

- manifest round-trip for chunk metadata
- chunk naming and row-range planning
- interrupted prepare leaves only `.tmp` files or incomplete manifest state
- loader skips already-loaded chunks with matching checksum
- loader rejects checksum mismatch
- loader rejects inconsistent load-state/table state

Integration smoke:

- prepare a small synthetic parquet fixture into chunks
- load chunks into PG18
- interrupt/resume by preloading the first chunk and verifying the second chunk
  loads without duplicating rows
- verify loaded row counts and basic index build can run after chunked load

Real-corpus validation:

- Prepare `ec_hnsw_real_ann_benchmarks_anchor` from the staged DBPedia parquet.
- Load it into PG18 with chunk-state resume enabled.
- Record disk usage, peak RSS if practical, and elapsed time in a review packet.

## Acceptance Criteria

- 990k/10k DBPedia fixture preparation no longer requires holding all selected
  vectors in memory at once.
- Loading can resume after interruption at chunk boundaries.
- Existing single-TSV prepare/load behavior is preserved.
- A completed chunked load produces the same table contract as the current
  loader: `<prefix>_corpus` and `<prefix>_queries`.
- Review packet includes raw prepare/load logs and manifest excerpts sufficient
  to audit chunk counts, row counts, and checksums.
