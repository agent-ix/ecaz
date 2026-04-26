# Artifact Manifest

## pg18_source_score_backlink_cache_concurrent_50k_timing.sql

- head SHA: `2b656612842b364ff772eb0e6183801753d940dd`
- packet/topic: `663-c1-concurrent-dsm-backlink-score-cache-timing`
- lane: PG18 source-scored concurrent DSM real 50k build timing after backlink target score-cache reuse
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing for this artifact
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.sql --log-output review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.log`
- timestamp: `2026-04-26T08:42:13-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with backlink-cache concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_backlink_cache_concurrent_50k_timing.sql`
  - `pg18_source_score_backlink_cache_concurrent_50k_timing.log`
- key result lines:
  - `corpus_rows = 50000`
  - `query_rows = 1000`
  - concurrent DSM backlink cache `CREATE INDEX Time: 197371.287 ms (03:17.371)`
  - `requested_workers = 4`
  - `workers_launched = 4`
  - `heap_tuples = 50000`
  - `index_tuples = 50000`
  - `heap_ingest_us = 28422134`
  - `graph_us = 165691168`
  - `stage_us = 2105372`
  - `write_us = 983173`
  - `concurrent_dsm_graph_workers_launched = 4`
  - `concurrent_dsm_backlink_cache_index_bytes = 68280320`

## pg18_source_score_backlink_cache_concurrent_50k_recall.sql

- head SHA: `2b656612842b364ff772eb0e6183801753d940dd`
- packet/topic: `663-c1-concurrent-dsm-backlink-score-cache-timing`
- lane: PG18 source-scored concurrent DSM real 50k recall check after backlink target score-cache reuse
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: external recall summary at `ef_search = 200`
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.sql --log-output review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.log`
- timestamp: `2026-04-26T08:45:35-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with existing serial baseline index and backlink-cache concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_backlink_cache_concurrent_50k_recall.sql`
  - `pg18_source_score_backlink_cache_concurrent_50k_recall.log`
- key result lines:
  - `serial_graph_recall_at_10 = 0.91`
  - `backlink_cache_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `backlink_cache_dsm_graph_recall_at_100 = 0.77`
  - `recall_100_delta = 0.007999957`
  - `backlink_cache_dsm_graph_below_exact_queries = 7`
  - `backlink_cache_dsm_worst_exact_gap = 2`
