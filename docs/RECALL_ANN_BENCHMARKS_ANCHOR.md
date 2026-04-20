# ann-benchmarks Reference Anchor

This document records the single published external recall result we anchor the
real-corpus A4 lane against. It exists so that, if `build_external_recall_context`
or `scripts/qdrant_dbpedia_to_tsv.py` ever silently corrupts the corpus, the
real-corpus gate runs do not become a self-referential measurement that nobody
can spot-check.

The anchor is **not a gate**. It is an oracle: a single number, on a real public
corpus, with a published source we can cite. If the local measurement drifts
more than `0.02` (2%) absolute from the published number, that is a signal that
the loader, the converter, the build path, or the scan path is broken — not a
signal to "tune" the local number.

## Anchor Row

| Field | Value |
| --- | --- |
| Source | Qdrant `vector-db-benchmark` results, `parallel=1` lane |
| Source URL | https://qdrant.tech/benchmarks/results-1-100-thread-2024-06-15.json |
| Engine | qdrant |
| Setup name | `qdrant-m-16-ef-128` |
| Dataset | `dbpedia-openai-1M-1536-angular` |
| Distance | cosine (== inner product on unit-normalized vectors) |
| Vector dim | 1536 |
| Corpus rows in published benchmark | 1,000,000 |
| Build `m` | 16 |
| Build `ef_construct` | 128 |
| Search `hnsw_ef` | 128 |
| Published `recall@10` | `0.96082` |
| Published `rps` (parallel=1) | `193.043` |

The published `recall@10` is copied verbatim from the
`mean_precisions` field of the matching JSON entry. The setup-name → search
parameter mapping is pinned by
https://raw.githubusercontent.com/qdrant/vector-db-benchmark/master/experiments/configurations/qdrant-single-node.json
under the `qdrant-m-16-ef-128` config: the `parallel=1` search lane sweeps
`hnsw_ef ∈ [64, 128, 256, 512]`, which matches the four parallel=1
JSON entries with recall `[0.94978, 0.96082, 0.96640, 0.96902]` in that
order. The second entry is our anchor.

The anchor constant lives in `src/lib.rs` as
`ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10` and must be updated in lockstep
with this document.

### Why this row and not another

This is the most widely reproduced public hnsw row at 1536-dim on an OpenAI
embedding corpus. The Qdrant benchmark is a published, source-controlled
fixture, the engine is hnsw with `m=16, ef_construct=128`, the search uses a
plain `hnsw_ef=128` with no quantization or rerank, and the dataset family is
the same DBpedia / OpenAI 1536-dim split that
`scripts/qdrant_dbpedia_to_tsv.py` already converts. Picking the
`hnsw_ef=128` row in particular matches the build path our gate already uses
(`ef_construction=128`) and uses an `ef_search` value that is already in the
A4 gate sweep, so the anchor reuses the existing build artifact rather than
forcing a separate index.

`hnswlib` and `FAISS-HNSW` rows on the same dataset would also be acceptable,
but Qdrant publishes their full configuration and a stable JSON results file
with cell-level recall numbers. ann-benchmarks reports against
`dbpedia-openai-1000k-angular` are a sibling reproduction of the same
underlying data and would yield the same anchor within tolerance, but their
results table is a moving target hosted on a Github Pages site without a
permalink.

## Caveats and equivalences

- **Distance metric.** The Qdrant benchmark dataset declares `distance:
  cosine`. Our local probe uses inner product on unit-normalized vectors. On
  unit-norm input these produce the same ranking, so `recall@10` is directly
  comparable. The loader (`scripts/load_real_corpus.py` `VectorNormStats`)
  logs mean / min / max L2 norm at load time; if the staged corpus is not
  unit-norm the operator will see a warning before the anchor is run.
- **Query split.** The published benchmark uses a separate held-out query
  set shipped alongside the 1M corpus tarball. The Qdrant Hugging Face
  parquet release we convert from has 1,000,000 rows total and ships no
  separate query split. The anchor profile (see below) instead reuses the
  same canonical sorted-id selection rule used by the rest of
  `docs/RECALL_REAL_CORPUS.md` and treats the last 10,000 rows of the parquet
  as the query set, leaving 990,000 rows for the corpus. This is an
  architectural reproduction, not a bit-for-bit one. The 0.02 tolerance below
  exists primarily to absorb this gap.
- **Build path.** The anchor must use the same `build_source_column = 'source'`
  hnsw build path as the primary gate
  (`docs/RECALL_REAL_CORPUS.md`). Building from the quantized column would
  invalidate the comparison.
- **Single row, no sweep.** This document deliberately records one
  `(m, ef_construct, hnsw_ef)` row. The gate sweep across `ef_search` is
  task 10054's responsibility, not the anchor's.

## How to reproduce

The anchor uses a dedicated converter profile so the staged TSVs and the
manifest are reproducible from the same parquet that the rest of the
real-corpus lane already loads.

1. Convert the parquet release into the anchor TSV pair plus manifest:
   ```bash
   python3 scripts/qdrant_dbpedia_to_tsv.py \
       --profile ec_hnsw_real_ann_benchmarks_anchor \
       --parquet /path/to/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data \
       --output-dir /path/to/staged
   ```
   This emits:
   - `ec_hnsw_real_ann_benchmarks_anchor_corpus.tsv` (990,000 rows)
   - `ec_hnsw_real_ann_benchmarks_anchor_queries.tsv` (10,000 rows)
   - `ec_hnsw_real_ann_benchmarks_anchor_manifest.json`
2. Load it. The loader is the same `scripts/load_real_corpus.py` used by the
   primary gate. Build the `m=16` index (other `m` values are not part of the
   anchor):
   ```bash
   PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \
       --prefix ec_hnsw_real_ann_benchmarks_anchor \
       --corpus-file /path/to/staged/ec_hnsw_real_ann_benchmarks_anchor_corpus.tsv \
       --queries-file /path/to/staged/ec_hnsw_real_ann_benchmarks_anchor_queries.tsv \
       --m 16
   ```
3. Run the anchor probe:
   ```sql
   SELECT *
   FROM ec_hnsw_graph_scan_recall_ann_benchmarks_reference(
       'ec_hnsw_real_ann_benchmarks_anchor_corpus',
       'ec_hnsw_real_ann_benchmarks_anchor_queries',
       'ec_hnsw_real_ann_benchmarks_anchor_m16_idx',
       16,
       128
   );
   ```
   The probe returns one row:
   ```
   m | ef_search | recall_at_10 | published_recall_at_10 | absolute_delta | within_two_percent
   16|       128 |        0.xxx |               0.96082  |          0.xxx | true|false
   ```
   `within_two_percent` is the only field a reviewer needs to look at. If it
   is `false`, do not "fix" it by adjusting the published constant. Land the
   probe red, capture the row in the review packet, and file a follow-up to
   investigate the loader, the converter, the build path, or the scan path.

The same flow is wrapped by an `#[ignore]`d Rust integration test in
`tests/recall_integration.rs` (`ann_benchmarks_anchor_within_tolerance`),
which drives convert + load + probe end-to-end as a single command. See the
test's doc comment for the environment variables it expects.

## What this document does not cover

- Continuous monitoring of the anchor. It is a one-time sanity check that
  reviewers can re-run when something feels off.
- Other published rows on the same dataset. Adding more rows turns the
  anchor into a sweep, which is task 10054's surface, not this one.
- The primary A4 gate. That stays on `ec_hnsw_real_50k` and
  `ec_hnsw_real_10k`; see `docs/RECALL_REAL_CORPUS.md`.
