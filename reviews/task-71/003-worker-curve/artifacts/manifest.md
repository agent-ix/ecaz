# Task 71 / Packet 003 Artifact Manifest

- Head SHA: `faa22c2c3`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/003-worker-curve/`
- Slice: Worker-curve suite setup plus callback-wiring validation,
  CLI-owned IVF parallel-build DB setup/probe, and fresh post-fix worker
  curve
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Surface: dry-run suite config for isolated prefixes per dataset/worker count;
  pre-fix hosted suite run; post-fix focused pg_test validation; suite runner
  per-load worker-counter support; fresh post-fix suite run on isolated
  one-index-per-table prefixes
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

### CLI-owned one-cell IVF parallel-build probe

- Install artifact: `install-current-ecaz-pg18-after-loader-timing.log`
- Install key lines:
  - `installed_backend=/opt/homebrew/lib/postgresql@18/ecaz.dylib`
  - `sha256=ee85182df636a8ef9819f633a9d076d571663c8b07b79990ecb2e939c8ca941b`
- Command:
  `./target/debug/ecaz dev test ivf-parallel-build-probe --host /Users/peter/.pgrx --port 28818 --drop-first`
- Result: passed without approval escalation
- Artifact: `probe-load-real10k-w2-after-loader-timing.log`
- Lane / fixture / storage / rerank:
  - lane: local PG18 one-cell probe
  - fixture: Task 31 staged real10k corpus/query TSVs loaded into isolated
    `task71_probe_w2` tables
  - storage format: `pq_fastscan`
  - rerank mode: `heap_f32`
- Surface: isolated one-index-per-table probe, not shared-table
- Key lines:
  - `built task71_probe_w2_idx in 433.37ms`
  - `requested_workers=2 workers_launched=2 heap_tuples=10000 index_tuples=10000`
  - `parallel_worker_tuple_buffer_capacity=16384`
- Notes:
  - This probe runs through `ecaz dev test`, which owns the DB setup/test
    surface and avoids the previous long inline shell/SQL command.
  - The result verifies IVF parallel build worker launch under the current
    installed PG18 dylib; it is not parallel scan evidence and is not a full
    worker-curve replacement.

### CLI-owned Task 71 matrix cleanup

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/003-worker-curve/artifacts/task71-clean-before-final-suite.log dev test ivf-parallel-build-clean --include-probe`
- Result: passed without approval escalation
- Artifact: `task71-clean-before-final-suite.log`
- Surface: drops isolated Task 71 matrix/probe table prefixes before the fresh
  suite run
- Key lines:
  - `[ivf-clean] dropped 17 prefixes`

### Fresh post-fix worker-curve suite

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/003-worker-curve/artifacts/suite-run-final.log bench suite run --config reviews/task-71/003-worker-curve/suite.json`
- Result: passed without approval escalation
- Artifacts:
  - `suite-run-final.log`
  - `suite-manifest.json`
  - `results.jsonl`
  - `load-real{10k,25k,50k,100k}-w{1,2,4,8}.log`
  - `recall-real{10k,25k,50k,100k}-w{1,2,4,8}.log`
  - `storage-real{10k,25k,50k,100k}-w{1,2,4,8}.log`
- Lane / fixture / storage / rerank:
  - lane: local PG18 M5 worker curve
  - fixture: Task 31 staged DBPedia real10k/25k/50k/100k corpus/query TSVs
  - storage format: `pq_fastscan`
  - rerank mode: `heap_f32`
- Surface: isolated one-index-per-table prefixes per scale/worker cell
- Key worker launch lines from `results.jsonl` / load-log
  `ec_ivf_build_timing` rows:
  - real10k: `1/1`, `2/2`, `4/4`, `8/7` requested/launched
  - real25k: `1/1`, `2/2`, `4/4`, `8/7` requested/launched
  - real50k: `1/1`, `2/2`, `4/4`, `8/7` requested/launched
  - real100k: `1/1`, `2/2`, `4/4`, `8/7` requested/launched
- Full build-index seconds:
  - real10k: w1 `0.464140`, w2 `0.436080`, w4 `0.414400`, w8 `0.411170`
  - real25k: w1 `0.721680`, w2 `0.652020`, w4 `0.621400`, w8 `0.612060`
  - real50k: w1 `1.160000`, w2 `1.020000`, w4 `0.937100`, w8 `0.922410`
  - real100k: w1 `2.630000`, w2 `2.220000`, w4 `2.070000`, w8 `2.030000`
- Best full-build speedups over w1:
  - real10k: ~1.13x at w8
  - real25k: ~1.18x at w8
  - real50k: ~1.26x at w8
  - real100k: ~1.30x at w8
- Heap-ingest timing does scale inside the parallel build path. For real100k,
  `heap_ingest_us` is w1 `877228`, w2 `497257`, w4 `322029`, w8 `274479`.
- Recall@10:
  - real10k: `1.0000` for all workers
  - real25k: `0.9990` for all workers
  - real50k: `1.0000` for all workers
  - real100k: `0.9820` for all workers
- ec_ivf index size invariance:
  - real10k: `2726298` bytes for all workers
  - real25k: `5557453` bytes for all workers
  - real50k: `10171187` bytes for all workers
  - real100k: `20342374` bytes for all workers
- Interpretation:
  - The suite validates parallel-build worker launch and recall/storage
    invariance.
  - The current implementation still does not satisfy Task 71's multi-x full
    build-time exit criterion; leader-side train/stage/flush work dominates
    after the parallel heap-ingest portion.

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
  - `load-real10k-w1 -> PGOPTIONS="-c max_parallel_maintenance_workers=1 -c max_parallel_workers=1" ... --table-reloption parallel_workers=1`
  - `load-real100k-w8 -> PGOPTIONS="-c max_parallel_maintenance_workers=8 -c max_parallel_workers=8" ... --table-reloption parallel_workers=8`
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
- The Task 31 baseline recall@10 points cited by `request.md` are:
  real10k `1.0000`, real25k `0.9990`, real50k `1.0000`, and directly
  comparable real100k n128/w500 `0.9820`. The adjacent Task 31 real100k
  n64/w750 fixed-scale point is `0.9940`.
- `allow_manifest_mismatch: true` is used only because the reused Task 31
  staged manifests carry `ec_hnsw_real_*` prefixes while the suite loads into
  isolated `task71_real*_w*` prefixes. The source corpus/query files remain
  the staged Task 31 TSVs under `data/task31_m5_dbpedia_staged/`.
- The fresh full suite regenerated `suite-manifest.json` and `results.jsonl`
  after the callback/config fixes. The dry-run artifacts remain the rendered
  config evidence for the suite shape.
