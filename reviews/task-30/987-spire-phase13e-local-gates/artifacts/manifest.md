# Task 30 / Packet 987 Artifact Manifest

- Head SHA: `908be140cc4734521913c3c7b282747e67262e67`
- Task bucket: `reviews/task-30/987-spire-phase13e-local-gates`
- Timestamp: `2026-05-26`
- Lane: local PG18 SPIRE Phase 13e production-read gates
- Fixture: one coordinator plus local remote PostgreSQL instances
- Storage format: `rabitq`
- Rerank mode: default SPIRE settings unless a fixture-specific script sets a GUC
- Isolation: local multi-instance fixtures; no AWS instances were used

## Artifacts

### `artifacts/core-suite-with-bench-suite/phase13e-local-gates-summary.tsv`

- Command:
  `bash scripts/run_spire_phase13e_local_gates_pg18.sh --suite core --skip-install --artifact-dir reviews/task-30/987-spire-phase13e-local-gates/artifacts/core-suite-with-bench-suite`
- Key result lines:
  - `phase13e-static-remote-placement pass`
  - `multicluster-customscan-read pass`
  - `insert-read-after-customscan-helper pass`
  - `insert-read-after-customscan-trigger pass`
  - `transport-overlap pass`

### `artifacts/core-suite-with-bench-suite/phase13e-static-remote-placement/phase13e-static-remote-placement.log`

- Command: produced by the core suite command above.
- Key result lines:
  - `plan=... Custom Scan (EcSpireDistributedScan) ... remote_fanout: 3`
  - `bench_suite_summary=passed|.../phase13e-local-spire-pipeline-suite.json|.../suite-manifest.json|.../results.jsonl`
  - `production_timeline_summary=3|3|623|26|0`
  - `strict_remote_failure_exit_code=3`
  - `degraded_profile_summary=degraded_ready|3|2|2|2|1|0|0|6|none`
  - `SPIRE Phase 13e static remote placement PG18 fixture passed`

### `artifacts/core-suite-with-bench-suite/phase13e-static-remote-placement/bench-suite/`

- Suite config:
  `phase13e-local-spire-pipeline-suite.json`
- Suite manifest:
  `suite-manifest.json`
- Structured results:
  `results.jsonl`
- Human-readable step log:
  `spire-pipeline.log`
- Command emitted by `ecaz bench suite`:
  `ecaz bench spire-pipeline --prefix ec_spire_phase13e_coord --index ec_spire_phase13e_coord_idx --queries-limit 1 --sweep 3 --include-remote --require-remote-placements --remote-selected-pids 2,3,4 --top-k 6 --consistency-mode strict --remote-tuple-transport pg_binary_attr_v1 --include-query-metrics --include-recall --include-production-read-profile --production-read-only --query-metric-k 6`
- Key `spire-pipeline.log` result lines:
  - `production_read_only: true`
  - `tuple_transport_status = ready`
  - `recall@k = 1.0000`
  - `status = ready`
  - `remote_pid_sum = 3`
  - `dispatch_sum = 3`
  - `socket_open_sum = 3`
  - `candidate_query_sum = 3`
  - `heap_query_sum = 3`
  - `returned_sum = 6`

### `artifacts/extended-suite-after-production-read/phase13e-local-gates-summary.tsv`

- Command:
  `bash scripts/run_spire_phase13e_local_gates_pg18.sh --suite extended --skip-install --artifact-dir reviews/task-30/987-spire-phase13e-local-gates/artifacts/extended-suite-after-production-read`
- Key result lines:
  - `stage-e-predispatch-epoch_mismatch pass`
  - `stage-e-predispatch-version_skew pass`
  - `stage-e-candidate-fingerprint_mismatch pass`
  - `stage-e-candidate-missing_or_reindexed_remote_index pass`
  - `stage-e-network-partition pass`
  - `stage-e-transport-connection_reset_mid_batch pass`
  - `stage-e-transport-local_cancel pass`
  - `stage-e-transport-local_statement_timeout pass`
  - `stage-e-transport-remote_backend_termination pass`
  - `stage-e-transport-remote_oom pass`
  - `stage-e-transport-remote_statement_timeout pass`
  - `stage-e-lifecycle-create_index_concurrently_missing_descriptor pass`
  - `stage-e-lifecycle-create_index_concurrently_new_descriptor pass`
  - `stage-e-lifecycle-drop_remote_index_before_fanout pass`
  - `stage-e-lifecycle-drop_remote_index_in_flight pass`
  - `stage-e-lifecycle-reindex_remote_index_before_fanout pass`
  - `stage-e-lifecycle-reindex_remote_index_in_flight pass`

### Focused CLI Tests

- Command: `cargo test --package ecaz-cli spire_pipeline`
- Result: `19 passed; 0 failed`
- Command: `cargo test --package ecaz-cli render_spire_registrations`
- Result: `7 passed; 0 failed`

### Build / Formatting Checks

- Command: `cargo build --release --package ecaz-cli`
- Result: finished release build; one pre-existing warning for `LoadedDistributedPlacementConfig::path`.
- Command: `cargo fmt`
- Result: completed; rustfmt emitted the repo's existing nightly-only config warnings.
- Command: `git diff --check`
- Result: no whitespace errors.
