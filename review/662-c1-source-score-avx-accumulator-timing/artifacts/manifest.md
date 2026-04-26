# Artifact Manifest

## pg18_source_score_avx_accum_concurrent_50k_timing.sql

- head SHA: `1802d707339a24785ee1cea46688a3e5b50c2056`
- packet/topic: `662-c1-source-score-avx-accumulator-timing`
- lane: PG18 source-scored concurrent DSM real 50k build timing after AVX source-score accumulator unroll
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing for this artifact
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/662-c1-source-score-avx-accumulator-timing/artifacts/pg18_source_score_avx_accum_concurrent_50k_timing.sql --log-output review/662-c1-source-score-avx-accumulator-timing/artifacts/pg18_source_score_avx_accum_concurrent_50k_timing.log`
- timestamp: `2026-04-26T08:20:58-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with AVX accumulator concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_avx_accum_concurrent_50k_timing.sql`
  - `pg18_source_score_avx_accum_concurrent_50k_timing.log`
- key result lines:
  - `corpus_rows = 50000`
  - `query_rows = 1000`
  - concurrent DSM AVX accumulator `CREATE INDEX Time: 204909.120 ms (03:24.909)`
  - `requested_workers = 4`
  - `workers_launched = 4`
  - `heap_tuples = 50000`
  - `index_tuples = 50000`
  - `heap_ingest_us = 28742619`
  - `graph_us = 172760957`
  - `stage_us = 2140793`
  - `write_us = 1019236`
  - `concurrent_dsm_graph_workers_launched = 4`
  - `concurrent_dsm_avx_accum_index_bytes = 68280320`

## pg18_source_score_avx_accum_concurrent_50k_recall.sql

- head SHA: `1802d707339a24785ee1cea46688a3e5b50c2056`
- packet/topic: `662-c1-source-score-avx-accumulator-timing`
- lane: PG18 source-scored concurrent DSM real 50k recall check after AVX source-score accumulator unroll
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: external recall summary at `ef_search = 200`
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/662-c1-source-score-avx-accumulator-timing/artifacts/pg18_source_score_avx_accum_concurrent_50k_recall.sql --log-output review/662-c1-source-score-avx-accumulator-timing/artifacts/pg18_source_score_avx_accum_concurrent_50k_recall.log`
- timestamp: `2026-04-26T08:24:29-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with existing serial baseline index and AVX accumulator concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_avx_accum_concurrent_50k_recall.sql`
  - `pg18_source_score_avx_accum_concurrent_50k_recall.log`
- key result lines:
  - `serial_graph_recall_at_10 = 0.91`
  - `avx_accum_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `avx_accum_dsm_graph_recall_at_100 = 0.774`
  - `recall_100_delta = 0.011999965`
  - `avx_accum_dsm_graph_below_exact_queries = 7`
  - `avx_accum_dsm_worst_exact_gap = 2`
