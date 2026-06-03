# Task 71 / Packet 003 Artifact Manifest

- Head SHA: `dcd45b2d8`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/003-worker-curve/`
- Slice: Worker-curve suite setup
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Surface: dry-run suite config for isolated prefixes per dataset/worker count
- Timestamp: 2026-06-02 America/Los_Angeles

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
- Result: failed with the previous suite shape before `table_reloptions`
- Key lines:
  - `building task71_real10k_w1_idx using ec_ivf`
  - `ERROR: unrecognized parameter "parallel_workers"`
  - `suite step "load-real10k-w1" failed with exit code 1`
