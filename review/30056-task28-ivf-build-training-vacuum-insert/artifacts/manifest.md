# Artifact Manifest

Packet: `30056-task28-ivf-build-training-vacuum-insert`

Head SHA: `43563e53c6ca6801e8136a1775ef99d27e4a8d0a`

Timestamp: `2026-04-27T15:24:00-07:00`

Lane: Task 28 IVF build/training/vacuum/insert deeper pass

Fixture: local PG18 scratch, database `postgres`, synthetic 4D `ecvector`
insert stress tables.

Storage format: `turboquant`

Rerank mode: `heap_f32`, `rerank_width=10` for the stress harness index.

Surface isolation: isolated one-index-per-table stress surfaces:
`task28_ivf_insert_smoke`, `task28_ivf_insert_c1`, and `task28_ivf_insert_c4`.

Cache state: warm local development run.

Memory high-water mark: not captured.

## Artifacts

- `list_ivf_functions.sql`, `list_ivf_functions.log`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30056-task28-ivf-build-training-vacuum-insert/artifacts/list_ivf_functions.sql --raw --log-output review/30056-task28-ivf-build-training-vacuum-insert/artifacts/list_ivf_functions.log`
  - Key line: installed scratch DB exposed only `public.ec_ivf_handler(internal)`, so stress logs used fallback relation stats instead of `ec_ivf_index_admin_snapshot`.

- `ivf_insert_c1_prefix_failure.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30056-task28-ivf-build-training-vacuum-insert/artifacts/ivf_insert_c1.log`
  - Key line: before fix commit `43563e5`, live insert failed with `ec_ivf list directory tuple length mismatch: got 80, expected 37`.

- `ivf_insert_smoke.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_smoke --seed-rows 64 --duration-seconds 2 --concurrency 1 --batch-rows 1 --nlists 8 --nprobe 8 --training-sample-rows 64 --log-output review/30056-task28-ivf-build-training-vacuum-insert/artifacts/ivf_insert_smoke.log`
  - Key lines:
    - `total_inserted_rows=476`
    - `inserted_rows_per_second=238.00`
    - `total_live_tuples=540`

- `ivf_insert_c1.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30056-task28-ivf-build-training-vacuum-insert/artifacts/ivf_insert_c1.log`
  - Key lines:
    - `total_inserted_rows=668`
    - `inserted_rows_per_second=66.80`
    - `total_live_tuples=1668`
    - `index_bytes=237568`

- `ivf_insert_c4.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30056-task28-ivf-build-training-vacuum-insert/artifacts/ivf_insert_c4.log`
  - Key lines:
    - `total_inserted_rows=1592`
    - `inserted_rows_per_second=159.20`
    - `total_live_tuples=2592`
    - `index_bytes=311296`

## Validation

- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
  - `39 passed; 0 failed`
- `cargo pgrx test pg18 test_ec_ivf_large_build_insert_directory_chain`
  - `1 passed; 0 failed`
- `cargo test -p ecaz-cli ivf_insert --no-default-features`
  - `3 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`
