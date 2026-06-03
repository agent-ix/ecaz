# Task 71 / Packet 003 Artifact Manifest

- Head SHA: `20d4db545`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/003-worker-curve/`
- Slice: Worker-curve suite setup plus callback-wiring validation
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Surface: dry-run suite config for isolated prefixes per dataset/worker count;
  pre-fix hosted suite run; post-fix focused pg_test validation; suite runner
  per-load worker-counter support
- Timestamp: 2026-06-03 America/Los_Angeles

## Artifacts

### `cargo-test-ecaz-cli-suite.log`

- Command:
  `cargo test -p ecaz-cli commands::bench::suite::tests:: > reviews/task-71/003-worker-curve/artifacts/cargo-test-ecaz-cli-suite.log 2>&1`
- Result: passed
- Key lines:
  - `test commands::bench::suite::tests::artifact_dir_templates_rewrite_load_step_paths ... ok`
  - `test commands::bench::suite::tests::load_step_pgoptions_flow_into_manifest_record ... ok`
  - `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 363 filtered out; finished in 0.00s`

### `cargo-test-ecaz-cli-suite-table-reloptions.log`

- Command:
  `cargo test -p ecaz-cli commands::bench::suite::tests:: > reviews/task-71/003-worker-curve/artifacts/cargo-test-ecaz-cli-suite-table-reloptions.log 2>&1`
- Result: passed
- Key lines:
  - `test commands::bench::suite::tests::load_step_pgoptions_flow_into_manifest_record ... ok`
  - `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 364 filtered out; finished in 0.00s`

### Focused suite tests after `capture_parallel_workers`

- Command:
  `cargo test -p ecaz-cli commands::bench::suite::tests:: > reviews/task-71/003-worker-curve/artifacts/cargo-test-ecaz-cli-suite-capture-parallel-workers.log 2>&1`
- Result: passed
- Artifact: `cargo-test-ecaz-cli-suite-capture-parallel-workers.log`
- Key lines:
  - `test commands::bench::suite::tests::parses_parallel_worker_counter_output ... ok`
  - `test commands::bench::suite::tests::parallel_worker_counter_emits_result_row ... ok`
  - `test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 364 filtered out`

### PG18 compile check after `capture_parallel_workers`

- Command:
  `cargo check --no-default-features --features pg18 > reviews/task-71/003-worker-curve/artifacts/cargo-check-pg18-capture-parallel-workers.log 2>&1`
- Result: passed
- Artifact: `cargo-check-pg18-capture-parallel-workers.log`
- Key lines:
  - `Finished dev profile`

### `cargo-test-ecaz-cli-load-table-reloptions.log`

- Command:
  `cargo test -p ecaz-cli commands::corpus::load::tests::table_reloption_set_clause_strips_create_table_prefix > reviews/task-71/003-worker-curve/artifacts/cargo-test-ecaz-cli-load-table-reloptions.log 2>&1`
- Result: passed
- Key lines:
  - `test commands::corpus::load::tests::table_reloption_set_clause_strips_create_table_prefix ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 399 filtered out; finished in 0.00s`

### `suite-dry-run.log`

- Command:
  `cargo run -p ecaz-cli -- --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-71/003-worker-curve/suite.json --dry-run --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-dry-run-manifest.json > reviews/task-71/003-worker-curve/artifacts/suite-dry-run.log 2>&1`
- Result: passed
- Key lines:
  - `wrote reviews/task-71/003-worker-curve/artifacts/suite-dry-run-manifest.json`
  - `load-real10k-w1 -> PGOPTIONS="-c max_parallel_maintenance_workers=1" ... --table-reloption parallel_workers=1`
  - `load-real100k-w8 -> PGOPTIONS="-c max_parallel_maintenance_workers=8" ... --table-reloption parallel_workers=8`
  - `recall-real100k-w8 -> ... --log-output reviews/task-71/003-worker-curve/artifacts/recall-real100k-w8.log`
  - `storage-real100k-w8 -> ... --log-file reviews/task-71/003-worker-curve/artifacts/storage-real100k-w8.log`

### `suite-dry-run-manifest.json`

- Command:
  emitted by the dry-run command above with `--manifest-output`
- Result: passed
- Key lines:
  - `load-real100k-w8` command records `--table-reloption parallel_workers=8`
  - `load-real100k-w8` record has `pgoptions: -c max_parallel_maintenance_workers=8`
  - expected artifacts for load/recall/storage resolve under
    `reviews/task-71/003-worker-curve/artifacts/`

### `preflight-db-extension.log`

- Command:
  `cargo run -p ecaz-cli -- dev sql --pg 18 --db tqvector_bench --socket-dir /Users/peter/.pgrx --sql "SELECT current_database(), extversion FROM pg_extension WHERE extname = 'ecaz'; SELECT amname FROM pg_am WHERE amname = 'ec_ivf';" --log-output reviews/task-71/003-worker-curve/artifacts/preflight-db-extension.log`
- Result: passed
- Key lines:
  - `tqvector_bench  0.1.1`
  - `ec_ivf`

### `suite-run.log`

- Command:
  `cargo run -p ecaz-cli -- bench suite run --config reviews/task-71/003-worker-curve/suite.json --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-manifest.json --results-output reviews/task-71/003-worker-curve/artifacts/results.jsonl > reviews/task-71/003-worker-curve/artifacts/suite-run.log 2>&1`
- Result: failed during local sandbox socket access
- Key lines:
  - `psql: error: connection to server on socket "/Users/peter/.pgrx/.s.PGSQL.28818" failed: Operation not permitted`
  - `suite step "parallel-workers-before" failed with exit code 1`

### `suite-run-escalated.log`

- Command:
  `cargo run -p ecaz-cli -- bench suite run --config reviews/task-71/003-worker-curve/suite.json --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-manifest.json --results-output reviews/task-71/003-worker-curve/artifacts/results.jsonl > reviews/task-71/003-worker-curve/artifacts/suite-run-escalated.log 2>&1`
- Result: failed because child load commands lacked explicit host/hostaddr
- Key lines:
  - `tqvector_bench  0`
  - `both host and hostaddr are missing`
  - `suite step "load-real10k-w1" failed with exit code 1`

### `suite-run-escalated-hosted.log`

- Command:
  `cargo run -p ecaz-cli -- --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-71/003-worker-curve/suite.json --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-manifest.json --results-output reviews/task-71/003-worker-curve/artifacts/results.jsonl > reviews/task-71/003-worker-curve/artifacts/suite-run-escalated-hosted.log 2>&1`
- Result: completed for the pre-callback-fix implementation; invalid as final
  Phase 3 evidence because worker counter stayed at zero
- Key lines:
  - `load-real10k-w1 -> PGOPTIONS="-c max_parallel_maintenance_workers=1" ... --table-reloption parallel_workers=1`
  - `load-real100k-w8 -> PGOPTIONS="-c max_parallel_maintenance_workers=8" ... --table-reloption parallel_workers=8`
  - `parallel-workers-after -> ... pg_stat_get_db_parallel_workers_launched(oid) ...`
  - `tqvector_bench	0`
  - `wrote reviews/task-71/003-worker-curve/artifacts/results.jsonl`

### `parallel-workers-before.log` / `parallel-workers-after.log`

- Command:
  raw suite steps around the hosted suite run querying
  `pg_stat_get_db_parallel_workers_launched(oid)`
- Result: completed; showed no PG parallel workers launched during the pre-fix
  hosted suite run
- Key lines:
  - Before: `tqvector_bench	0`
  - After: `tqvector_bench	0`

### `parallel-settings-after-zero-workers.log`

- Command:
  `cargo run -p ecaz-cli -- --host /Users/peter/.pgrx --port 28818 dev sql --pg 18 --db tqvector_bench --sql "SHOW max_parallel_workers; SHOW max_parallel_maintenance_workers; SHOW max_worker_processes;" --log-output reviews/task-71/003-worker-curve/artifacts/parallel-settings-after-zero-workers.log`
- Result: passed
- Key lines:
  - `max_parallel_workers = 8`
  - `max_parallel_maintenance_workers = 2`
  - `max_worker_processes = 8`

### `cargo-pgrx-test-pg18-ivf-parallel-build-after-routine-callbacks.log`

- Command:
  `cargo pgrx test pg18 test_ec_ivf_parallel_build_workers_and_counts > reviews/task-71/003-worker-curve/artifacts/cargo-pgrx-test-pg18-ivf-parallel-build-after-routine-callbacks.log 2>&1`
- Result: failed during pgrx test setup, before the test body ran
- Key lines:
  - `failed writing ... ecaz.control ... Operation not permitted`
  - `Could not initialize test framework`

### `cargo-pgrx-test-pg18-ivf-parallel-build-after-routine-callbacks-escalated.log`

- Command:
  `cargo pgrx test pg18 test_ec_ivf_parallel_build_workers_and_counts > reviews/task-71/003-worker-curve/artifacts/cargo-pgrx-test-pg18-ivf-parallel-build-after-routine-callbacks-escalated.log 2>&1`
- Result: passed after the IVF routine callback fix
- Key lines:
  - `test tests::pg_test_ec_ivf_parallel_build_workers_and_counts ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1939 filtered out; finished in 34.38s`

## Current Suite Config

- `suite.json` now sets both PostgreSQL build-worker caps on each load step:
  `-c max_parallel_maintenance_workers=N -c max_parallel_workers=N`.
- `suite.json` still uses `--table-reloption parallel_workers=N` for the heap
  table storage option.
- `suite.json` now sets `capture_parallel_workers: true` on every load step.
  The post-fix run should write `parallel_workers_before`,
  `parallel_workers_after`, and `parallel_workers_delta` into each load
  `suite-manifest.json` record, plus `metric=parallel_workers` rows into
  `results.jsonl`.
- The next full suite run must regenerate `suite-dry-run.log`,
  `suite-dry-run-manifest.json`, `suite-manifest.json`, and `results.jsonl`
  after this config change before packet 003 can be used as final Phase 3
  evidence.
