# Artifact Manifest

Packet: `review/30064-task28-ivf-insert-dimension-harness`

Measured code SHA: `656b2dc` (`ecaz: parameterize IVF insert stress dimensions`)

Timestamp: `2026-04-27T16:16:27-07:00`

Lane: Task 28 IVF live-insert measurement harness quality, local PG18 only.

Fixture: synthetic `ecaz stress ivf-insert`, fresh local PG18 database
`task28_ivf_fresh_20260427`, 128 seed rows, 1 second insert window, batch size
1, `nlists=16`, `nprobe=16`, training sample rows 128, `--dimensions 1536`,
`--require-admin-snapshot`.

Storage format: IVF one posting tuple per inserted row, one-index-per-table
isolated stress surface.

Rerank mode: not applicable to insert stress.

Cache state: normal local PG18 process state; no explicit cache drop or warmup
outside the harness.

Memory high-water mark: not captured by this harness.

## Artifacts

### `ivf_insert_dim1536_smoke.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_dim1536_smoke --seed-rows 128 --duration-seconds 1 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 128 --dimensions 1536 --require-admin-snapshot --log-output review/30064-task28-ivf-insert-dimension-harness/artifacts/ivf_insert_dim1536_smoke.log`
- Key result lines cited:
  - `duration_seconds = 1`
  - `dimensions = 1536`
  - `total_inserted_rows = 146`
  - `inserted_rows_per_second = 146.00`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `inserted_since_build = 146`
  - `changed_row_fraction = 0.532847`
  - `average_list_live_count = 17.12`
  - `max_list_live_count = 62`
  - `list_imbalance_ratio = 3.620438`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
