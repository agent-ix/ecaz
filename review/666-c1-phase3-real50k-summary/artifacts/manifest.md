# Artifact Manifest

## pg18_concurrent_dsm_source_real_50k_rerun.sql / .log

- copied from: `review/658-c1-concurrent-dsm-source-real-50k-rerun/artifacts/`
- head SHA: `50290ad` lineage before later optimization packets
- packet/topic: `666-c1-phase3-real50k-summary`
- lane: PG18 source-scored concurrent DSM real 50k recall parity
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: external recall summary at `ef_search = 200`
- command used: copied source packet command from packet 658
- timestamp: copied artifact from packet 658
- isolated one-index-per-table or shared-table surface: shared real 50k table with serial baseline index and concurrent DSM sidecar index
- key result lines:
  - `serial_graph_recall_at_10 = 0.91`
  - `concurrent_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `concurrent_dsm_graph_recall_at_100 = 0.771`

## pg18_source_dsm_real_50k_build_timing.sql / .log

- copied from: `review/659-c1-source-dsm-real-50k-build-timing/artifacts/`
- head SHA: `50290ad` lineage before later optimization packets
- packet/topic: `666-c1-phase3-real50k-summary`
- lane: PG18 source-scored serial and concurrent DSM real 50k build timing before AVX/source-score optimizations
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing for this artifact
- command used: copied source packet command from packet 659
- timestamp: copied artifact from packet 659
- isolated one-index-per-table or shared-table surface: shared real 50k table with serial and concurrent DSM sidecar indexes
- key result lines:
  - serial source-scored `CREATE INDEX Time: 1815962.457 ms (30:15.962)`
  - serial source-scored `graph_us = 1784269081`
  - pre-optimization concurrent DSM `CREATE INDEX Time: 431268.704 ms (07:11.269)`
  - pre-optimization concurrent DSM `graph_us = 399932406`

## pg18_source_score_backlink_cache_concurrent_50k_timing.sql / .log

- copied from: `review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/`
- head SHA: `2b656612842b364ff772eb0e6183801753d940dd`
- packet/topic: `666-c1-phase3-real50k-summary`
- lane: PG18 source-scored concurrent DSM real 50k current-best build timing
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing for this artifact
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.sql --log-output review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.log`
- timestamp: `2026-04-26T08:42:13-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with backlink-cache concurrent DSM sidecar index
- key result lines:
  - current best concurrent DSM `CREATE INDEX Time: 197371.287 ms (03:17.371)`
  - `requested_workers = 4`
  - `workers_launched = 4`
  - `heap_tuples = 50000`
  - `index_tuples = 50000`
  - `graph_us = 165691168`
  - `concurrent_dsm_graph_workers_launched = 4`

## pg18_source_score_backlink_cache_concurrent_50k_recall.sql / .log

- copied from: `review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/`
- head SHA: `2b656612842b364ff772eb0e6183801753d940dd`
- packet/topic: `666-c1-phase3-real50k-summary`
- lane: PG18 source-scored concurrent DSM real 50k current-best recall check
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; first 10 queries evaluated
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: external recall summary at `ef_search = 200`
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.sql --log-output review/663-c1-concurrent-dsm-backlink-score-cache-timing/artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.log`
- timestamp: `2026-04-26T08:45:35-07:00`
- isolated one-index-per-table or shared-table surface: shared real 50k table with serial baseline index and backlink-cache concurrent DSM sidecar index
- key result lines:
  - `serial_graph_recall_at_10 = 0.91`
  - `backlink_cache_dsm_graph_recall_at_10 = 0.91`
  - `recall_delta = 0`
  - `serial_graph_recall_at_100 = 0.762`
  - `backlink_cache_dsm_graph_recall_at_100 = 0.77`
  - `recall_100_delta = 0.007999957`
