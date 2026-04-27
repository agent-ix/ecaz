# Artifact Manifest

Packet: `review/30060-task28-ivf-insert-normalize-once`

Measured code SHA: `647abd1` (`ivf: avoid duplicate insert normalization`)

Initial timestamp: `2026-04-27T15:53:32-07:00`

Fresh-database rerun timestamp: `2026-04-27T15:57:34-07:00`

Lane: Task 28 IVF live-insert fixed per-row work, local PG18 only.

Fixture: synthetic `ecaz stress ivf-insert`, 1000 seed rows, 10 second insert
window, batch size 1, `nlists=16`, `nprobe=16`, training sample rows 1000.

Storage format: IVF one posting tuple per inserted row, one-index-per-table
isolated stress surface.

Rerank mode: not applicable to insert stress.

Cache state: normal local PG18 process state; no explicit cache drop or warmup
outside the harness. The first two runs used the existing `postgres` scratch
database and fell back to relation stats because the current admin snapshot
function was not present in that database's extension object set. The fresh
reruns used `task28_ivf_fresh_20260427`, created with current extension SQL, and
captured `ec_ivf_index_admin_snapshot`.

Memory high-water mark: not captured by this harness.

## Artifacts

### `ivf_insert_normonce_c1.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_normonce_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30060-task28-ivf-insert-normalize-once/artifacts/ivf_insert_normonce_c1.log`
- Key result lines cited:
  - `total_inserted_rows = 2610`
  - `inserted_rows_per_second = 261.00`
  - `index_bytes = 376832`
  - `ivf insert stress harness passed`

### `ivf_insert_normonce_c4.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_normonce_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30060-task28-ivf-insert-normalize-once/artifacts/ivf_insert_normonce_c4.log`
- Key result lines cited:
  - `total_inserted_rows = 6497`
  - `inserted_rows_per_second = 649.70`
  - `index_bytes = 753664`
  - `ivf insert stress harness passed`

### `ivf_insert_normonce_fresh_c1.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_normonce_fresh_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30060-task28-ivf-insert-normalize-once/artifacts/ivf_insert_normonce_fresh_c1.log`
- Key result lines cited:
  - `total_inserted_rows = 2732`
  - `inserted_rows_per_second = 273.20`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 393216`
  - `inserted_since_build = 2732`
  - `changed_row_fraction = 0.732047`
  - `average_list_live_count = 233.25`
  - `max_list_live_count = 320`
  - `list_imbalance_ratio = 1.371919`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`

### `ivf_insert_normonce_fresh_c4.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_normonce_fresh_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30060-task28-ivf-insert-normalize-once/artifacts/ivf_insert_normonce_fresh_c4.log`
- Key result lines cited:
  - `total_inserted_rows = 6562`
  - `inserted_rows_per_second = 656.20`
  - `snapshot_source = ec_ivf_index_admin_snapshot`
  - `index_bytes = 770048`
  - `inserted_since_build = 6562`
  - `changed_row_fraction = 0.867760`
  - `average_list_live_count = 472.62`
  - `max_list_live_count = 621`
  - `list_imbalance_ratio = 1.313938`
  - `reindex_reason = changed_rows`
  - `ivf insert stress harness passed`
