# Artifact Manifest

Packet: `review/30065-task28-ivf-insert-dim1536-baseline`

Measured code SHA: `ed80057`

Timestamp: `2026-04-27T16:18:39-07:00`

Lane: Task 28 IVF live-insert dimension split, local PG18 only.

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

### `ivf_insert_dim1536_c1.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_dim1536_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --dimensions 1536 --require-admin-snapshot --log-output review/30065-task28-ivf-insert-dim1536-baseline/artifacts/ivf_insert_dim1536_c1.log`
- Key result lines cited:
  - `duration_seconds = 10`
  - `concurrency = 1`
  - `dimensions = 1536`
  - `total_inserted_rows = 1243`
  - `inserted_rows_per_second = 124.30`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 2236416`
  - `inserted_since_build = 1243`
  - `changed_row_fraction = 0.554169`
  - `average_list_live_count = 140.19`
  - `max_list_live_count = 210`
  - `list_imbalance_ratio = 1.497994`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_dim1536_c4.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_dim1536_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --dimensions 1536 --require-admin-snapshot --log-output review/30065-task28-ivf-insert-dim1536-baseline/artifacts/ivf_insert_dim1536_c4.log`
- Key result lines cited:
  - `duration_seconds = 10`
  - `concurrency = 4`
  - `dimensions = 1536`
  - `total_inserted_rows = 3936`
  - `inserted_rows_per_second = 393.60`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 4898816`
  - `inserted_since_build = 3936`
  - `changed_row_fraction = 0.797407`
  - `average_list_live_count = 308.50`
  - `max_list_live_count = 413`
  - `list_imbalance_ratio = 1.338736`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
