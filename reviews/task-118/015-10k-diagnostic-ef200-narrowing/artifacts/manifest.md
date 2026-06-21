# Task 118 Packet 015 Artifact Manifest

- head SHA: `607cdecbbb5ea43c4fad497c0e7ec7d8fce61710`
- task bucket: `reviews/task-118/015-10k-diagnostic-ef200-narrowing`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: Task 118 suite config narrowing for 10k HNSW diagnostic-only steps.
- isolated surface: dry-run only; no benchmark matrix run.

## Artifacts

### `suite-dry-run-10k-frontier-ef200.log`

- command: `cargo run -p ecaz-cli -- --log-file reviews/task-118/015-10k-diagnostic-ef200-narrowing/artifacts/suite-dry-run-10k-frontier-ef200.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/015-10k-diagnostic-ef200-narrowing/artifacts/suite-manifest-dry-run-10k-frontier-ef200.json --results-output reviews/task-118/015-10k-diagnostic-ef200-narrowing/artifacts/results-dry-run-10k-frontier-ef200.jsonl --only frontier-10k-hnsw-turboquant --only frontier-10k-hnsw-pq-fastscan --only frontier-10k-hnsw-rabitq --only frontier-10k-hnsw-turboquant-compressed-build --only frontier-10k-hnsw-pq-fastscan-compressed-build --only frontier-10k-hnsw-rabitq-compressed-build --dry-run --allow-debug-backend`
- result: passed; selected six `hnsw-frontier` steps, each expanded with `--sweep 200 --queries-limit 200`.

### `suite-manifest-dry-run-10k-frontier-ef200.json`

- purpose: machine-readable dry-run proof for the six selected 10k frontier steps.
- selected-step count: `hnsw-frontier	6`

### `suite-dry-run-10k-score-correlation-ef200.log`

- command: `cargo run -p ecaz-cli -- --log-file reviews/task-118/015-10k-diagnostic-ef200-narrowing/artifacts/suite-dry-run-10k-score-correlation-ef200.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/015-10k-diagnostic-ef200-narrowing/artifacts/suite-manifest-dry-run-10k-score-correlation-ef200.json --results-output reviews/task-118/015-10k-diagnostic-ef200-narrowing/artifacts/results-dry-run-10k-score-correlation-ef200.jsonl --only score-correlation-10k-hnsw-turboquant --only score-correlation-10k-hnsw-pq-fastscan --only score-correlation-10k-hnsw-rabitq --only score-correlation-10k-hnsw-turboquant-compressed-build --only score-correlation-10k-hnsw-pq-fastscan-compressed-build --only score-correlation-10k-hnsw-rabitq-compressed-build --dry-run --allow-debug-backend`
- result: passed; selected six `hnsw-score-correlation` steps, each expanded with `--sweep 200 --queries-limit 200`.

### `suite-manifest-dry-run-10k-score-correlation-ef200.json`

- purpose: machine-readable dry-run proof for the six selected 10k score-correlation steps.
- selected-step count: `hnsw-score-correlation	6`

## Config Check

- `jq empty crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`
  passed before the dry-runs.

## Closeout Impact

This narrows only Task 118 10k diagnostic-only steps. Recall and latency keep
their full sweeps. The final Task 118 decision table uses `ef_search=200` rows,
so this aligns 10k diagnostics with the already-narrowed 50k/100k diagnostic
shape and reduces final current-head regeneration cost.
