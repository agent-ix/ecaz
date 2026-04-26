# Artifact Manifest

## pg18_source_score_scratchbuf_concurrent_50k_timing.sql

- head SHA: `28c64c699c639bad8ce040c9ee7ecac0fb324982`
- packet/topic: `661-c1-concurrent-dsm-scratch-buffer-measurement`
- lane: PG18 source-scored concurrent DSM real 50k build timing after successor scratch-buffer reuse
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing for this artifact
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/661-c1-concurrent-dsm-scratch-buffer-measurement/artifacts/pg18_source_score_scratchbuf_concurrent_50k_timing.sql --log-output review/661-c1-concurrent-dsm-scratch-buffer-measurement/artifacts/pg18_source_score_scratchbuf_concurrent_50k_timing.log`
- timestamp: `2026-04-26T07:58:08-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with successor-scratch-buffer concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_scratchbuf_concurrent_50k_timing.sql`
  - `pg18_source_score_scratchbuf_concurrent_50k_timing.log`
- key result lines:
  - `corpus_rows = 50000`
  - `query_rows = 1000`
  - concurrent DSM scratch-buffer `CREATE INDEX Time: 209737.611 ms (03:29.738)`
  - `requested_workers = 4`
  - `workers_launched = 4`
  - `heap_tuples = 50000`
  - `index_tuples = 50000`
  - `heap_ingest_us = 29856557`
  - `graph_us = 176719376`
  - `stage_us = 2065738`
  - `write_us = 911123`
  - `concurrent_dsm_graph_workers_launched = 4`
  - `concurrent_dsm_scratchbuf_index_bytes = 68280320`

## pg18_source_score_scratchbuf_concurrent_50k_recall.sql

- head SHA: `28c64c699c639bad8ce040c9ee7ecac0fb324982`
- packet/topic: `661-c1-concurrent-dsm-scratch-buffer-measurement`
- lane: PG18 source-scored concurrent DSM real 50k recall check after successor scratch-buffer reuse
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: external recall summary at `ef_search = 200`
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/661-c1-concurrent-dsm-scratch-buffer-measurement/artifacts/pg18_source_score_scratchbuf_concurrent_50k_recall.sql --log-output review/661-c1-concurrent-dsm-scratch-buffer-measurement/artifacts/pg18_source_score_scratchbuf_concurrent_50k_recall.log`
- timestamp: `2026-04-26T08:01:42-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with existing serial baseline index and successor-scratch-buffer concurrent DSM sidecar index
- artifact files:
  - `pg18_source_score_scratchbuf_concurrent_50k_recall.sql`
  - `pg18_source_score_scratchbuf_concurrent_50k_recall.log`
- key result lines:
  - `serial_graph_recall_at_10 = 0.91`
  - `scratchbuf_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `scratchbuf_dsm_graph_recall_at_100 = 0.768`
  - `recall_100_delta = 0.0059999824`
  - `scratchbuf_dsm_graph_below_exact_queries = 7`
  - `scratchbuf_dsm_worst_exact_gap = 2`
