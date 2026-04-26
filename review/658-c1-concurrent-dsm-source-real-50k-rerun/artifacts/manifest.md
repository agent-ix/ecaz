# Artifact Manifest

## pg18_concurrent_dsm_source_real_50k_rerun.sql

- head SHA: `8f7bd896842777f02e331354bbcd7c02687c49d5`
- packet/topic: `658-c1-concurrent-dsm-source-real-50k-rerun`
- lane: PG18 source-scored concurrent DSM real 50k rerun
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: source-scored graph build; external recall summary at `ef_search = 200`
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/658-c1-concurrent-dsm-source-real-50k-rerun/artifacts/pg18_concurrent_dsm_source_real_50k_rerun.sql --log-output review/658-c1-concurrent-dsm-source-real-50k-rerun/artifacts/pg18_concurrent_dsm_source_real_50k_rerun.log`
- timestamp: `2026-04-25T23:26:39-07:00`
- isolated one-index-per-table or shared-table surface: shared table with serial `m16` source-scored baseline index and concurrent DSM source-scored sidecar index
- artifact files:
  - `pg18_concurrent_dsm_source_real_50k_rerun.sql`
  - `pg18_concurrent_dsm_source_real_50k_rerun.log`
- key result lines:
  - `serial_index_bytes = 68280320`
  - `requested_workers = 4`
  - `workers_launched = 4`
  - `heap_tuples = 50000`
  - `index_tuples = 50000`
  - `heap_ingest_us = 28202651`
  - `graph_us = 401643816`
  - `stage_us = 2099800`
  - `write_us = 951416`
  - `concurrent_dsm_graph_workers_launched = 4`
  - `concurrent_dsm_index_bytes = 68280320`
  - `serial_graph_recall_at_10 = 0.91`
  - `concurrent_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `concurrent_dsm_graph_recall_at_100 = 0.771`
  - `recall_100_delta = 0.009000003`
