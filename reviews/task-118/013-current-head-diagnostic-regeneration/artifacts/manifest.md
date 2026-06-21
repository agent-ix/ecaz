# Task 118 Packet 013 Artifact Manifest

- head SHA: `325241cd6544f004a44968262e22b14be5b3397f`
- task bucket: `reviews/task-118/013-current-head-diagnostic-regeneration`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: current-head closeout supplement for Task 118 HNSW frontier diagnostics.
- isolated surface: dry-run only; no benchmark matrix run.

## Artifacts

### `current-head-diagnostic-regeneration.md`

- purpose: operator supplement requiring current-head regeneration of 10k HNSW frontier diagnostics after packet 012 changed diagnostic semantics.
- includes: exact 10k frontier regeneration command, expected selected steps, final packet commit scope, and raw JSONL exclusion rule.

### `suite-dry-run-10k-frontier-current-head.log`

- command: `cargo run -p ecaz-cli -- --log-file reviews/task-118/013-current-head-diagnostic-regeneration/artifacts/suite-dry-run-10k-frontier-current-head.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/013-current-head-diagnostic-regeneration/artifacts/suite-manifest-dry-run-10k-frontier-current-head.json --results-output reviews/task-118/013-current-head-diagnostic-regeneration/artifacts/results-dry-run-10k-frontier-current-head.jsonl --only frontier-10k-hnsw-turboquant --only frontier-10k-hnsw-pq-fastscan --only frontier-10k-hnsw-rabitq --only frontier-10k-hnsw-turboquant-compressed-build --only frontier-10k-hnsw-pq-fastscan-compressed-build --only frontier-10k-hnsw-rabitq-compressed-build --dry-run --allow-debug-backend`
- result: passed; selected six `hnsw-frontier` steps.

### `suite-manifest-dry-run-10k-frontier-current-head.json`

- purpose: machine-readable dry-run proof for the six selected 10k frontier steps.
- selected-step count: `hnsw-frontier	6`
