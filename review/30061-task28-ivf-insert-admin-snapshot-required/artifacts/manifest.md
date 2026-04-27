# Artifact Manifest

Packet: `review/30061-task28-ivf-insert-admin-snapshot-required`

Measured code SHA: `4303b4c` (`ecaz: require IVF admin snapshot for insert stress`)

Timestamp: `2026-04-27T15:59:56-07:00`

Lane: Task 28 IVF live-insert measurement harness quality, local PG18 only.

Fixture: synthetic `ecaz stress ivf-insert`, fresh local PG18 database
`task28_ivf_fresh_20260427`, 1000 seed rows, 1 second insert window, batch size
1, `nlists=16`, `nprobe=16`, training sample rows 1000,
`--require-admin-snapshot`.

Storage format: IVF one posting tuple per inserted row, one-index-per-table
isolated stress surface.

Rerank mode: not applicable to insert stress.

Cache state: normal local PG18 process state; no explicit cache drop or warmup
outside the harness.

Memory high-water mark: not captured by this harness.

## Artifacts

### `ivf_insert_require_admin_smoke.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_require_admin_smoke --seed-rows 1000 --duration-seconds 1 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --require-admin-snapshot --log-output review/30061-task28-ivf-insert-admin-snapshot-required/artifacts/ivf_insert_require_admin_smoke.log`
- Key result lines cited:
  - `total_inserted_rows = 397`
  - `inserted_rows_per_second = 397.00`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `inserted_since_build = 397`
  - `changed_row_fraction = 0.284180`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
