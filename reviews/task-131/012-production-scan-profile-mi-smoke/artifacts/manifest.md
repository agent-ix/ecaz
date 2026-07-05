# Task 131 Packet 012 Artifact Manifest

- head SHA: `7425bf051bb3db157c7ab62de4e738754f0adc56`
- task bucket: `reviews/task-131/`
- packet path: `reviews/task-131/012-production-scan-profile-mi-smoke/`
- timestamp: `2026-07-01T08:49:01-07:00`
- lane: local multi-instance PG18, four local PostgreSQL instances
- fixture: Phase 13e static remote placement smoke fixture, 12 coordinator rows, 3 remote nodes
- storage format: `rabitq`
- index shape: smoke fixture `nlists=3 / nprobe=3`
- rerank mode: `rerank_width=0`
- isolation: local four-instance coordinator + three remotes, not one-index-per-table

## Commands

### Current CLI Build

```sh
cargo build -p ecaz-cli
```

### Local Multi-Instance Smoke

The first sandboxed attempt failed at `cargo pgrx install` because `/home/peter/.pgrx/...` is outside the writable sandbox. The successful run used the same command outside the sandbox so pgrx could install the current extension into PG18:

```sh
ECAZ_BIN=/home/peter/dev/ecaz/target/debug/ecaz \
  scripts/run_spire_phase13e_static_remote_placement_pg18.sh \
  --artifact-dir reviews/task-131/012-production-scan-profile-mi-smoke/artifacts \
  --run-id task131-scan-profile-mi-012c \
  --fixture-rows 12 \
  --bench-top-k 6 \
  --bench-queries-limit 1 \
  --bench-sweep 3
```

The harness generated and ran:

```sh
/home/peter/dev/ecaz/target/debug/ecaz \
  --database postgres \
  --host /home/peter/dev/ecaz/target/spire-phase13e-sockets-task131-scan-profile-mi-012c \
  --port 39440 \
  --user postgres \
  bench spire-pipeline \
  --prefix ec_spire_phase13e_coord \
  --index ec_spire_phase13e_coord_idx \
  --queries-limit 1 \
  --sweep 3 \
  --include-remote \
  --require-remote-placements \
  --remote-selected-pids 2,3,4 \
  --top-k 6 \
  --consistency-mode strict \
  --remote-tuple-transport pg_binary_attr_v1 \
  --include-query-metrics \
  --include-recall \
  --include-production-read-profile \
  --production-read-only \
  --query-metric-k 6 \
  --log-output reviews/task-131/012-production-scan-profile-mi-smoke/artifacts/bench-suite/spire-pipeline.log
```

## Artifacts

- `artifacts/phase13e-static-remote-placement.log`: local multi-instance harness log.
- `artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json`: generated `ecaz bench suite` config.
- `artifacts/bench-suite/suite-manifest.json`: generated suite manifest; step status is `succeeded`.
- `artifacts/bench-suite/results.jsonl`: structured result rows, including the new scan-profile rows.
- `artifacts/bench-suite/spire-pipeline.log`: rendered CLI report with `Production selected-leaf scan profile`.
- `artifacts/bench-suite/suite-run.log`: suite stdout/stderr, including rendered report.
- `artifacts/cargo-build-ecaz-cli.log`: current CLI build passed with the existing `LoadedDistributedPlacementConfig::path` dead-code warning.
- `artifacts/cargo-test-ecaz-cli-production-scan-profile.log`: focused render test passed.
- `artifacts/cargo-test-ecaz-cli-sql-contracts.log`: focused SQL-contract test passed.
- `artifacts/*postgres.log`, `node-*-materialize-*.log`, `production-read-timeline.tsv`, `strict-remote-node2-failure.log`, `slow-remote-node2-lock.log`: small harness support logs.

No corpus TSVs, SSM logs, tunnel state, or raw polling snapshots are included.

## Key Result Lines

From `artifacts/phase13e-static-remote-placement.log`:

- `bench_suite_summary=passed|reviews/task-131/012-production-scan-profile-mi-smoke/artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json|reviews/task-131/012-production-scan-profile-mi-smoke/artifacts/bench-suite/suite-manifest.json|reviews/task-131/012-production-scan-profile-mi-smoke/artifacts/bench-suite/results.jsonl`
- `SPIRE Phase 13e static remote placement PG18 fixture passed`

From `artifacts/bench-suite/results.jsonl`:

- query metrics: recall@6 `1.0000`, latency p50/p95/p99 `62.287 ms`
- production read profile: `status=ready`, `remote_pid_sum=3`, `dispatch_sum=3`, `compact_candidate_sum=12`, `remote_heap_candidate_sum=12`, `strict_fail_sum=0`, `timeout_sum=0`, `cancel_sum=0`, `degraded_skip_sum=0`, `returned_sum=6`
- scan profile node 2: `selected_pid_sum=1`, `scanned_pid_sum=1`, `leaf_candidate_sum=6`, `winner_sum=6`, `sound_bound_available_sum=0`, `sound_bound_missing_sum=1`, `local_kth_count=1`, `local_kth_min=-0.627586`, `local_kth_max=-0.627586`
- scan profile node 3: `selected_pid_sum=1`, `scanned_pid_sum=1`, `leaf_candidate_sum=3`, `winner_sum=3`, `sound_bound_available_sum=0`, `sound_bound_missing_sum=1`, `local_kth_count=0`
- scan profile node 4: `selected_pid_sum=1`, `scanned_pid_sum=1`, `leaf_candidate_sum=3`, `winner_sum=3`, `sound_bound_available_sum=0`, `sound_bound_missing_sum=1`, `local_kth_count=0`

## Defect Found And Fixed

The first multi-instance suite attempt after packet 011 panicked in the CLI while decoding the production scan-profile row:

- `error retrieving column 1: error deserializing column 1`
- location: `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs:3679`

The cause was a CLI decoder mismatch: `ec_spire_remote_search_production_scan_profile` exposes `node_id` as SQL `bigint`/Rust `i64`, while the CLI decoded it as `i32`. Commit `7425bf051bb3db157c7ab62de4e738754f0adc56` fixes `ProductionScanProfileRow::node_id` and `ProductionScanProfileKey::node_id` to `i64`. The rerun succeeded and produced the scan-profile rows above.
