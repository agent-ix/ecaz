# Artifact Manifest

Packet: `review/30063-task28-ivf-insert-single-posting-encode`

Measured code base SHA before trial backout: `ab624c3`

Timestamp: `2026-04-27T16:11:33-07:00`

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

### `ivf_insert_singleposting_c1.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_singleposting_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --require-admin-snapshot --log-output review/30063-task28-ivf-insert-single-posting-encode/artifacts/ivf_insert_singleposting_c1.log`
- Key result lines cited:
  - `total_inserted_rows = 2678`
  - `inserted_rows_per_second = 267.80`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `inserted_since_build = 2678`
  - `changed_row_fraction = 0.728113`
  - `average_list_live_count = 229.88`
  - `max_list_live_count = 297`
  - `list_imbalance_ratio = 1.292007`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_singleposting_c4.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_singleposting_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --require-admin-snapshot --log-output review/30063-task28-ivf-insert-single-posting-encode/artifacts/ivf_insert_singleposting_c4.log`
- Key result lines cited:
  - `total_inserted_rows = 6502`
  - `inserted_rows_per_second = 650.20`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `inserted_since_build = 6502`
  - `changed_row_fraction = 0.866702`
  - `average_list_live_count = 468.88`
  - `max_list_live_count = 593`
  - `list_imbalance_ratio = 1.264729`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
