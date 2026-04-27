# Artifact Manifest

Packet: `review/30066-task28-ivf-insert-assign-without-normalize`

Measured code SHA: `302ee783c33b70f8c887d25edabf0553225e685a`

Timestamp: `2026-04-27T16:28:43-07:00`

Lane: Task 28 IVF live-insert assignment cleanup, local PG18 only.

Fixture: synthetic `ecaz stress ivf-insert`, fresh local PG18 database
`task28_ivf_fresh_20260427`, 1000 seed rows, 10 second insert window, batch
size 1, `nlists=16`, `nprobe=16`, training sample rows 1000,
`--dimensions 1536`, `--require-admin-snapshot`.

Storage format: IVF one posting tuple per inserted row, one-index-per-table
isolated stress surface.

Rerank mode: not applicable to insert stress.

Cache state: normal local PG18 process state; no explicit cache drop or warmup
outside the harness.

Memory high-water mark: not captured by this harness.

## Artifacts

### `ivf_insert_assignraw_dim1536_c1.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_assignraw_dim1536_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --dimensions 1536 --require-admin-snapshot --log-output review/30066-task28-ivf-insert-assign-without-normalize/artifacts/ivf_insert_assignraw_dim1536_c1.log`
- Key result lines cited:
  - `duration_seconds = 10`
  - `concurrency = 1`
  - `dimensions = 1536`
  - `total_inserted_rows = 1228`
  - `inserted_rows_per_second = 122.80`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 2220032`
  - `inserted_since_build = 1228`
  - `changed_row_fraction = 0.551167`
  - `average_list_live_count = 139.25`
  - `max_list_live_count = 194`
  - `list_imbalance_ratio = 1.393178`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_assignraw_dim1536_c1_r2.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_assignraw_dim1536_c1_r2 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --dimensions 1536 --require-admin-snapshot --log-output review/30066-task28-ivf-insert-assign-without-normalize/artifacts/ivf_insert_assignraw_dim1536_c1_r2.log`
- Key result lines cited:
  - `duration_seconds = 10`
  - `concurrency = 1`
  - `dimensions = 1536`
  - `total_inserted_rows = 1261`
  - `inserted_rows_per_second = 126.10`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 2252800`
  - `inserted_since_build = 1261`
  - `changed_row_fraction = 0.557718`
  - `average_list_live_count = 141.31`
  - `max_list_live_count = 198`
  - `list_imbalance_ratio = 1.401150`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_assignraw_dim1536_c4.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_assignraw_dim1536_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --dimensions 1536 --require-admin-snapshot --log-output review/30066-task28-ivf-insert-assign-without-normalize/artifacts/ivf_insert_assignraw_dim1536_c4.log`
- Key result lines cited:
  - `duration_seconds = 10`
  - `concurrency = 4`
  - `dimensions = 1536`
  - `total_inserted_rows = 4068`
  - `inserted_rows_per_second = 406.80`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 5038080`
  - `inserted_since_build = 4068`
  - `changed_row_fraction = 0.802684`
  - `average_list_live_count = 316.75`
  - `max_list_live_count = 427`
  - `list_imbalance_ratio = 1.348066`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_assignraw_dim1536_c4_r2.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_assignraw_dim1536_c4_r2 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --dimensions 1536 --require-admin-snapshot --log-output review/30066-task28-ivf-insert-assign-without-normalize/artifacts/ivf_insert_assignraw_dim1536_c4_r2.log`
- Key result lines cited:
  - `duration_seconds = 10`
  - `concurrency = 4`
  - `dimensions = 1536`
  - `total_inserted_rows = 4030`
  - `inserted_rows_per_second = 403.00`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 4997120`
  - `inserted_since_build = 4030`
  - `changed_row_fraction = 0.801193`
  - `average_list_live_count = 314.38`
  - `max_list_live_count = 404`
  - `list_imbalance_ratio = 1.285089`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
