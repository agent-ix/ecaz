# Artifact Manifest: Concurrent DSM Real 990k Scale

## pg18_real990k_m16_w8_build.sql / .log

- head SHA: `8703f51d6e4b`
- packet/topic: `669-c1-concurrent-dsm-real990k-scale`
- lane: PG18 source-scored concurrent DSM real-990k m16 8-worker build
- fixture: `/home/peter/dev/datasets/tqhnsw_real_ann_benchmarks_anchor_chunked`; 990,000 corpus rows x 1536 dimensions plus 10,000 query rows
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: source-scored build surface; no recall query in this packet
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/669-c1-concurrent-dsm-real990k-scale/artifacts/pg18_real990k_m16_w8_build.sql --log-output review/669-c1-concurrent-dsm-real990k-scale/artifacts/pg18_real990k_m16_w8_build.log`
- timestamp: `2026-04-26T14:25:32-07:00` to `2026-04-26T15:56:59-07:00`
- isolated one-index-per-table or shared-table surface: shared 990k table with one sidecar m16/w8 index

Cluster settings:

- `max_worker_processes = 16`
- `max_parallel_workers = 16`
- `max_parallel_maintenance_workers = 8`
- table reloption: `parallel_workers = 8`

Key result lines:

- fixture: `corpus_rows = 990000`; `query_rows = 10000`; chunk state `corpus = 20 / 990000`, `queries = 1 / 10000`
- `CREATE INDEX` time: `5517326.098 ms (01:31:57.326)`
- `real990k_m16_w8 | requested_workers = 8 | workers_launched = 8 | heap_workers_launched = 0 | graph_workers_launched = 8 | graph_us = 4656361521`
- `heap_ingest_us = 598095013`; `flush_total_us = 4892094823`; `stage_us = 209065560`; `write_us = 24934225`
- `concurrent_dsm_graph_workers_launched = 8`
- index size: `1351688192` bytes

## load_real990k_chunked_m16.log

- head SHA: `8703f51d6e4b`
- packet/topic: `669-c1-concurrent-dsm-real990k-scale`
- lane: PG18 chunked load for real-990k fixture
- fixture: chunked corpus manifest `/home/peter/dev/datasets/tqhnsw_real_ann_benchmarks_anchor_chunked/ec_hnsw_real_ann_benchmarks_anchor_manifest.json`
- command used: `script --quiet --return --flush --log-out review/669-c1-concurrent-dsm-real990k-scale/artifacts/load_real990k_chunked_m16.log --command "cargo run -p ecaz-cli -- --database postgres corpus load --prefix ec_hnsw_real_ann_benchmarks_anchor --profile ec_hnsw --manifest-file /home/peter/dev/datasets/tqhnsw_real_ann_benchmarks_anchor_chunked/ec_hnsw_real_ann_benchmarks_anchor_manifest.json --chunked --m 16 --host /home/peter/.pgrx --port 28818"`
- timestamp: `2026-04-26T13:18:00-07:00` to `2026-04-26T14:24:49-07:00`
- isolated one-index-per-table or shared-table surface: shared 990k table; loader automatic index build was terminated before completion because it did not set the controlled parallel-worker SQL knobs

Key result lines:

- loaded corpus chunks `corpus-00000.tsv` through `corpus-00019.tsv`
- loaded query chunk `queries-00000.tsv`
- automatic loader index build started for `ec_hnsw_real_ann_benchmarks_anchor_m16_idx`, then exited with `connection closed` after the backend was terminated intentionally

Notes:

- `prepare_real990k.log` is an earlier interrupted one-shot prepare attempt and is not cited as a successful artifact.
