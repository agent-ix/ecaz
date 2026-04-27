# Artifact Manifest

Packet: `30057-task28-ivf-insert-hotpath`

Head SHA: `bfbb40d199ed289f9e015789a2a0ea6a364cd946`

Timestamp: `2026-04-27T15:45:00-07:00`

Lane: Task 28 IVF live-insert hot-path follow-up

Fixture: local PG18 scratch, database `postgres`, synthetic 4D `ecvector`
insert stress tables.

Storage format: `turboquant`

Rerank mode: `heap_f32`, `rerank_width=10` for the stress harness index.

Surface isolation: isolated one-index-per-table stress surfaces:
`task28_ivf_insert_listcheck_c1` and `task28_ivf_insert_listcheck_c4`.

Cache state: warm local development run.

Memory high-water mark: not captured.

## Artifacts

- `ivf_insert_listcheck_c1.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_listcheck_c1 --seed-rows 1000 --duration-seconds 10 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30057-task28-ivf-insert-hotpath/artifacts/ivf_insert_listcheck_c1.log`
  - Key lines:
    - `total_inserted_rows=2753`
    - `inserted_rows_per_second=275.30`
    - `total_live_tuples=3753`
    - `index_bytes=393216`

- `ivf_insert_listcheck_c4.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres stress ivf-insert --table task28_ivf_insert_listcheck_c4 --seed-rows 1000 --duration-seconds 10 --concurrency 4 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --log-output review/30057-task28-ivf-insert-hotpath/artifacts/ivf_insert_listcheck_c4.log`
  - Key lines:
    - `total_inserted_rows=6575`
    - `inserted_rows_per_second=657.50`
    - `total_live_tuples=7575`
    - `index_bytes=819200`

## Validation

- `cargo test --lib am::ec_ivf::insert --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
  - `39 passed; 0 failed`
- `cargo pgrx test pg18 test_ec_ivf_insert`
  - `6 passed; 0 failed`
- `cargo pgrx test pg18 test_ec_ivf_large_build_insert_directory_chain`
  - `1 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`
