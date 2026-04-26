# Artifact Manifest: Concurrent DSM W8 Headroom

## pg18_concurrent_dsm_real50k_w8_headroom.sql / .log

- head SHA: `509692d33ed7`
- packet/topic: `672-c1-concurrent-dsm-w8-headroom`
- lane: PG18 source-scored concurrent DSM real-50k 8-worker headroom diagnostic
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, loaded under prefix `tqhnsw_real_50k_reloaded`; 50,000 corpus rows x 1536 dimensions
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: source-scored build surface; no recall query in this packet
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/672-c1-concurrent-dsm-w8-headroom/artifacts/pg18_concurrent_dsm_real50k_w8_headroom.sql --log-output review/672-c1-concurrent-dsm-w8-headroom/artifacts/pg18_concurrent_dsm_real50k_w8_headroom.log`
- timestamp: `2026-04-26T11:20:19-07:00` to `2026-04-26T11:22:46-07:00`
- isolated one-index-per-table or shared-table surface: shared real-50k table with one sidecar index for this 8-worker setting

Cluster settings:

- `max_worker_processes = 16`
- `max_parallel_workers = 16`
- `max_parallel_maintenance_workers = 8`

Key result lines:

- `concurrent_dsm_source_w8_headroom | requested_workers = 8 | workers_launched = 8 | heap_workers_launched = 0 | graph_workers_launched = 8 | graph_us = 116850823`
- `CREATE INDEX` time for 8 workers with headroom: `147947.506 ms (02:27.948)`
- index size: `68280320` bytes

Notes:

- Packet 668 recorded the previous 8-worker request under `max_worker_processes = 8`; it launched only 7 graph workers and took `04:08.671`.
- This packet updates the 8-worker conclusion after raising PG18 worker-process headroom.
