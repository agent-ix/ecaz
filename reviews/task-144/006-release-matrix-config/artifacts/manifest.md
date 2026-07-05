# Task 144 Packet 006 Artifact Manifest

- head SHA: `8c7cdeea5040085b3e7e9d9257e9efc38df3d8e4`
- task bucket: `reviews/task-144/`
- packet path: `reviews/task-144/006-release-matrix-config/`
- timestamp: `2026-07-05T09:51:50-07:00`
- slice: pre-registered Task 144 release matrix suite config for closure assignment plus query-time ratio pruning.
- build profile: dry-run/audit only; the config includes a release backend precheck step that records `ecaz_build_profile()` when the real suite is run.
- isolated one-index-per-table vs shared-table: one index prefix per scale/assignment variant; no shared-table matrix shape.

## Artifacts

### `suite-task144-release-matrix.json`

- `ecaz bench suite` config.
- Matrix shape:
  - scales: `10k`, `50k`, `100k`
  - assignment variants: `single`, `fixed_b2`, `closure_e010_b8`
  - query modes: `fixed`, `ratio125`, `adaptive`
  - total steps: 46
  - step kinds: 1 raw precheck, 9 load, 9 storage, 27 `spire-pipeline`
- Evidence outputs planned:
  - storage logs for each loaded shape
  - `spire-pipeline` logs with `include_recall`, `include_query_metrics`, and `include_production_read_profile`
  - stage containment JSONL for per-query route/probed-list and recall-tail rows
  - result identity JSONL for returned-id auditing

### `suite-audit.log`

- command:
  `target/release/ecaz bench suite audit --config reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json`
- result:
  `audit passed: 46 steps`

### `suite-dry-run.log`

- command:
  `target/release/ecaz bench suite run --config reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json --dry-run --manifest-output reviews/task-144/006-release-matrix-config/artifacts/suite-dry-run-manifest.json --results-output reviews/task-144/006-release-matrix-config/artifacts/results-dry-run.jsonl`
- result:
  command exit code 0
- key dry-run checks:
  - 46 dry-run steps
  - 27 `spire-pipeline` cells
  - 9 cells with `ec_spire.probe_distance_ratio=1.25`
  - 9 cells with `--adaptive-nprobe`

### `suite-dry-run-manifest.json`

- Expanded command manifest from the dry-run.
- All 46 steps are selected and have status `dry-run`.

## Notes

This packet does not run the long release matrix and does not claim Task 144 closeout. It makes the closeout run concrete and reviewable before spending the runtime. The real matrix still needs to be executed on a release PG18 backend, then summarized from packet-local `results.jsonl`, pipeline logs, storage logs, stage containment JSONL, and result identity JSONL.
