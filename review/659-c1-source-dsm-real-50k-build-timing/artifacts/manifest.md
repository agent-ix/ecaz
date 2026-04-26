# Artifact Manifest

## pg18_source_dsm_real_50k_build_timing.sql

- head SHA: `fade320874c980a35bbae60abd534cdb7428d34b`
- packet/topic: `659-c1-source-dsm-real-50k-build-timing`
- lane: PG18 source-scored real 50k serial vs concurrent DSM build timing
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: build-only timing, no recall query in this packet
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/659-c1-source-dsm-real-50k-build-timing/artifacts/pg18_source_dsm_real_50k_build_timing.sql --log-output review/659-c1-source-dsm-real-50k-build-timing/artifacts/pg18_source_dsm_real_50k_build_timing.log`
- timestamp: `2026-04-26T07:21:02-07:00`
- isolated one-index-per-table or shared-table surface: shared table with existing serial baseline index plus serial timing sidecar and concurrent DSM timing sidecar
- artifact files:
  - `pg18_source_dsm_real_50k_build_timing.sql`
  - `pg18_source_dsm_real_50k_build_timing.log`
- key result lines:
  - `corpus_rows = 50000`
  - `query_rows = 1000`
  - `existing_serial_index_bytes = 68280320`
  - serial `CREATE INDEX Time: 1815961.793 ms (30:15.962)`
  - serial `requested_workers = 4`
  - serial `workers_launched = 0`
  - serial `heap_tuples = 50000`
  - serial `index_tuples = 50000`
  - serial `heap_ingest_us = 28805318`
  - serial `graph_us = 1784269081`
  - serial `stage_us = 1824645`
  - serial `write_us = 946879`
  - serial `serial_timing_index_bytes = 68280320`
  - concurrent DSM `CREATE INDEX Time: 431268.704 ms (07:11.269)`
  - concurrent DSM `requested_workers = 4`
  - concurrent DSM `workers_launched = 4`
  - concurrent DSM `heap_tuples = 50000`
  - concurrent DSM `index_tuples = 50000`
  - concurrent DSM `heap_ingest_us = 28227765`
  - concurrent DSM `graph_us = 399932406`
  - concurrent DSM `stage_us = 2032188`
  - concurrent DSM `write_us = 939198`
  - concurrent DSM `concurrent_dsm_timing_index_bytes = 68280320`
