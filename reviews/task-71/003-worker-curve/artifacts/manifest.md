# Task 71 / Packet 003 Artifact Manifest

- Head SHA: `0bb998345`
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

### `suite-dry-run.log`

- Command:
  `cargo run -p ecaz-cli -- bench suite run --config reviews/task-71/003-worker-curve/suite.json --dry-run --manifest-output reviews/task-71/003-worker-curve/artifacts/suite-dry-run-manifest.json > reviews/task-71/003-worker-curve/artifacts/suite-dry-run.log 2>&1`
- Result: passed
- Key lines:
  - `wrote reviews/task-71/003-worker-curve/artifacts/suite-dry-run-manifest.json`
  - `load-real10k-w1 -> PGOPTIONS="-c max_parallel_maintenance_workers=1"`
  - `load-real100k-w8 -> PGOPTIONS="-c max_parallel_maintenance_workers=8"`
  - `recall-real100k-w8 -> ... --log-output reviews/task-71/003-worker-curve/artifacts/recall-real100k-w8.log`
  - `storage-real100k-w8 -> ... --log-file reviews/task-71/003-worker-curve/artifacts/storage-real100k-w8.log`

### `suite-dry-run-manifest.json`

- Command:
  emitted by the dry-run command above with `--manifest-output`
- Result: passed
- Key lines:
  - `load-real100k-w8` command records `parallel_workers=8`
  - `load-real100k-w8` record has `pgoptions: -c max_parallel_maintenance_workers=8`
  - expected artifacts for load/recall/storage resolve under
    `reviews/task-71/003-worker-curve/artifacts/`
