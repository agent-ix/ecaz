# Artifact Manifest

## pg18_source_score_avx_concurrent_50k_timing.sql

- head SHA: `d9b79567c9402e254ff193f7bb4abb5995cb2c26`
- packet/topic: `660-c1-source-score-avx-build-timing`
- lane: PG18 source-scored concurrent DSM real 50k build timing after AVX source-score kernel
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing for this artifact
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/660-c1-source-score-avx-build-timing/artifacts/pg18_source_score_avx_concurrent_50k_timing.sql --log-output review/660-c1-source-score-avx-build-timing/artifacts/pg18_source_score_avx_concurrent_50k_timing.log`
- timestamp: `2026-04-26T07:46:56-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with AVX concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_avx_concurrent_50k_timing.sql`
  - `pg18_source_score_avx_concurrent_50k_timing.log`
- key result lines:
  - `corpus_rows = 50000`
  - `query_rows = 1000`
  - concurrent DSM AVX `CREATE INDEX Time: 207594.528 ms (03:27.595)`
  - `requested_workers = 4`
  - `workers_launched = 4`
  - `heap_tuples = 50000`
  - `index_tuples = 50000`
  - `heap_ingest_us = 29400628`
  - `graph_us = 174810922`
  - `stage_us = 2123382`
  - `write_us = 1058449`
  - `concurrent_dsm_graph_workers_launched = 4`
  - `concurrent_dsm_avx_index_bytes = 68280320`

## pg18_source_score_avx_concurrent_50k_recall.sql

- head SHA: `d9b79567c9402e254ff193f7bb4abb5995cb2c26`
- packet/topic: `660-c1-source-score-avx-build-timing`
- lane: PG18 source-scored concurrent DSM real 50k recall check after AVX source-score kernel
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: external recall summary at `ef_search = 200`
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/660-c1-source-score-avx-build-timing/artifacts/pg18_source_score_avx_concurrent_50k_recall.sql --log-output review/660-c1-source-score-avx-build-timing/artifacts/pg18_source_score_avx_concurrent_50k_recall.log`
- timestamp: `2026-04-26T07:46:56-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with existing serial baseline index and AVX concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_avx_concurrent_50k_recall.sql`
  - `pg18_source_score_avx_concurrent_50k_recall.log`
- key result lines:
  - `serial_graph_recall_at_10 = 0.91`
  - `avx_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `avx_dsm_graph_recall_at_100 = 0.772`
  - `recall_100_delta = 0.00999999`
  - `avx_dsm_graph_below_exact_queries = 7`
  - `avx_dsm_worst_exact_gap = 2`
