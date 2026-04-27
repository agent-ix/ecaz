# Artifact Manifest

Packet: `review/30059-task28-ivf-insert-centroid-cache`

Measured code SHA: `f2314bb` (`ivf: cache insert centroid model`)

Backout SHA: `ce7a2b0` (`ivf: back out insert centroid cache`)

Timestamp: `2026-04-27T15:39:07-07:00`

Lane: Task 28 IVF live-insert fixed per-row work, local PG18 only.

Fixture: synthetic `ecaz stress ivf-insert`, 1000 seed rows, 10 second insert
window, `batch_rows=1`, `nlists=16`, `nprobe=16`,
`training_sample_rows=1000`.

Storage / rerank: default `ec_ivf` stress surface, isolated one-index-per-table.

## 1 Worker

- File: `ivf_insert_centroidcache_c1.log`
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_centroidcache_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30059-task28-ivf-insert-centroid-cache/artifacts/ivf_insert_centroidcache_c1.log`
- Surface isolation: isolated one-index-per-table.
- Key lines:
  - `total_inserted_rows = 2496`
  - `inserted_rows_per_second = 249.60`
  - `total_live_tuples = 3496`
  - `index_bytes = 376832`

## 4 Workers

- File: `ivf_insert_centroidcache_c4.log`
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_centroidcache_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30059-task28-ivf-insert-centroid-cache/artifacts/ivf_insert_centroidcache_c4.log`
- Surface isolation: isolated one-index-per-table.
- Key lines:
  - `total_inserted_rows = 6355`
  - `inserted_rows_per_second = 635.50`
  - `total_live_tuples = 7355`
  - `index_bytes = 753664`
