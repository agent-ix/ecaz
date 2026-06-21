# Task 118 Packet 008 Artifact Manifest

- head SHA: `4e124d14ff923dee6b63720ad6fe910700305307`
- task bucket: `reviews/task-118/008-large-scale-diagnostic-ef200`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: Task 118 HNSW large-scale suite dry-run for 50k and 100k source-build and compressed-build lanes across TurboQuant, PqFastScan, and RaBitQ.
- isolated surface: one HNSW index per loaded prefix in the suite config.

## Artifacts

### `suite-dry-run-50k-100k-diagnostic-ef200.log`

- command:
  `cargo run -p ecaz-cli -- --log-file reviews/task-118/008-large-scale-diagnostic-ef200/artifacts/suite-dry-run-50k-100k-diagnostic-ef200.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/008-large-scale-diagnostic-ef200/artifacts --manifest-output reviews/task-118/008-large-scale-diagnostic-ef200/artifacts/suite-manifest-dry-run-50k-100k-diagnostic-ef200.json --only-tag ec_real_50k --only-tag ec_real_100k --dry-run --allow-debug-backend`
- purpose: dry-run expansion of the 50k/100k final Task 118 suite lanes after narrowing large-scale diagnostic sweeps.
- key result: all selected `hnsw-frontier` and `hnsw-score-correlation` commands expand with `--sweep 200 --queries-limit 200`.

### `suite-dry-run-50k-100k-diagnostic-ef200.stdout`

- command: stdout/stderr capture from the same dry-run command.
- purpose: durable expansion log for review.
- key result: latency commands still expand with `--sweep "40,64,100,128,160,200"` while diagnostic commands use `--sweep 200`.

### `suite-manifest-dry-run-50k-100k-diagnostic-ef200.json`

- command: produced by the dry-run command above.
- purpose: machine-readable selected-step manifest.
- selected step counts:
  - `hnsw-frontier`: 12
  - `hnsw-score-correlation`: 12
  - `recall`: 12
  - `latency`: 12
  - `load`: 12
  - `storage`: 12

### `dry-run-diagnostic-sweep-summary.txt`

- command:
  `jq` summary over `suite-manifest-dry-run-50k-100k-diagnostic-ef200.json`.
- purpose: compact proof that every selected 50k/100k diagnostic step uses `--sweep 200 --queries-limit 200`.
