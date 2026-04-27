# Artifact Manifest

Packet: `review/30062-task28-ivf-insert-combined-stats-wal`

Measured code base SHA before trial backout: `7f1d1a4`

Timestamp: `2026-04-27T16:06:31-07:00`

Lane: Task 28 IVF live-insert fixed per-row work, local PG18 only.

Fixture: synthetic `ecaz stress ivf-insert`, fresh local PG18 database
`task28_ivf_fresh_20260427`, 1000 seed rows, 10 second insert window, batch
size 1, `nlists=16`, `nprobe=16`, training sample rows 1000,
`--require-admin-snapshot`.

Storage format: IVF one posting tuple per inserted row, one-index-per-table
isolated stress surface.

Rerank mode: not applicable to insert stress.

Cache state: normal local PG18 process state; no explicit cache drop or warmup
outside the harness.

Memory high-water mark: not captured by this harness.

## Artifacts

### `ivf_insert_combinedstats_c1.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_combinedstats_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --require-admin-snapshot --log-output review/30062-task28-ivf-insert-combined-stats-wal/artifacts/ivf_insert_combinedstats_c1.log`
- Key result lines cited:
  - `total_inserted_rows = 2652`
  - `inserted_rows_per_second = 265.20`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `inserted_since_build = 2652`
  - `changed_row_fraction = 0.726177`
  - `average_list_live_count = 228.25`
  - `max_list_live_count = 297`
  - `list_imbalance_ratio = 1.301205`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_combinedstats_c4.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_combinedstats_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --require-admin-snapshot --log-output review/30062-task28-ivf-insert-combined-stats-wal/artifacts/ivf_insert_combinedstats_c4.log`
- Key result lines cited:
  - `total_inserted_rows = 6451`
  - `inserted_rows_per_second = 645.10`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `inserted_since_build = 6451`
  - `changed_row_fraction = 0.865790`
  - `average_list_live_count = 465.69`
  - `max_list_live_count = 618`
  - `list_imbalance_ratio = 1.327070`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
