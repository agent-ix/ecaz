# Artifact Manifest: Concurrent DSM Worker Sweep

## pg18_concurrent_dsm_real50k_worker_sweep.sql / .log

- head SHA: `01dc5831e720017d4b488b361804096a2b1ac998`
- packet/topic: `668-c1-concurrent-dsm-worker-sweep`
- lane: PG18 source-scored concurrent DSM real-50k worker sweep
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: source-scored build surface; no recall query in this packet
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/668-c1-concurrent-dsm-worker-sweep/artifacts/pg18_concurrent_dsm_real50k_worker_sweep.sql --log-output review/668-c1-concurrent-dsm-worker-sweep/artifacts/pg18_concurrent_dsm_real50k_worker_sweep.log`
- timestamp: `2026-04-26T10:04:46-07:00` to `2026-04-26T10:24:26-07:00`
- isolated one-index-per-table or shared-table surface: shared real-50k table with one sidecar index per worker setting

Key result lines:

- `concurrent_dsm_source_w1 | requested_workers = 1 | workers_launched = 1 | graph_us = 395621949`
- `CREATE INDEX` time for 1 worker: `432016.708 ms (07:12.017)`
- `concurrent_dsm_source_w2 | requested_workers = 2 | workers_launched = 2 | graph_us = 268137745`
- `CREATE INDEX` time for 2 workers: `299789.517 ms (04:59.790)`
- `concurrent_dsm_source_w4 | requested_workers = 4 | workers_launched = 4 | graph_us = 173200231`
- `CREATE INDEX` time for 4 workers: `204964.443 ms (03:24.964)`
- `concurrent_dsm_source_w8 | requested_workers = 8 | workers_launched = 7 | graph_us = 216938590`
- `CREATE INDEX` time for 8 requested workers: `248671.461 ms (04:08.671)`
