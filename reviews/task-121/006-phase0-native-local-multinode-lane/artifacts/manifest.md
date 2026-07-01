# Task 121 Phase 0 Native Local Multinode Lane Artifacts

- head_sha: `69a644d9091e678ea1e58fca7dad461eb79103bc`
- task_bucket: `reviews/task-121`
- packet: `reviews/task-121/006-phase0-native-local-multinode-lane`
- scope: local-only native four-instance PG18 multinode lane for Task 121 Phase 0
- timestamp: `2026-06-23T12:38:04Z`
- lane: local multinode correctness smoke
- fixture: generated synthetic 10k corpus, 100 generated queries
- storage format: `rabitq`
- rerank mode: remote heap candidate production read profile
- isolated surfaces: yes; four separate local PG18 instances on one host, with independent coordinator and remote indexes
- AWS usage: none

## Validation Artifacts

### `validation/cargo-test-ecaz-cli-suite.log`

- command: `script -q -c "cargo test -p ecaz-cli commands::bench::suite" reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/validation/cargo-test-ecaz-cli-suite.log`
- result: PASS
- key lines:
  - `running 54 tests`
  - `test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 360 filtered out`
  - `COMMAND_EXIT_CODE="0"`

### `validation/cargo-build-ecaz-cli.log`

- command: `script -q -c "cargo build -p ecaz-cli --bin ecaz" reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/validation/cargo-build-ecaz-cli.log`
- result: PASS
- key lines:
  - `warning: field path is never read` in `crates/ecaz-cli/src/commands/corpus/load.rs`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.21s`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase0-native-local-multinode-dryrun.json`

- command config for a suite-driven native local multinode matrix cell.
- storage format: `rabitq`
- shared reloptions: `nlists=128`, `recursive_fanout=8`, `top_graph_enabled=1`
- coordinator reloptions: `training_sample_rows=10000`
- remote reloptions: `boundary_replica_count=1`

### `validation/suite-phase0-native-local-multinode-dryrun.script.log`

- command: `script -q -c "target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/suite-phase0-native-local-multinode-dryrun.json --manifest-output reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/suite-phase0-native-local-multinode-dryrun-manifest.json --results-output reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/suite-phase0-native-local-multinode-dryrun-results.jsonl --log-file reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/validation/suite-phase0-native-local-multinode-dryrun.log" reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/validation/suite-phase0-native-local-multinode-dryrun.script.log`
- result: PASS
- key generated command includes:
  - `dev spire-multicluster local-multinode-pg18`
  - `--storage-format rabitq`
  - `--coord-index task121_native_coord_idx`
  - `--remote-index task121_native_remote_idx`
  - `--reloption nlists=128`
  - `--coord-reloption training_sample_rows=10000`
  - `--remote-reloption boundary_replica_count=1`
  - `--skip-bench-suite`
  - `COMMAND_EXIT_CODE="0"`

### `suite-phase0-native-local-multinode-dryrun-manifest.json`

- generated manifest for the dry-run suite config.
- expected artifacts with `skip_bench_suite=true`:
  - `reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/dryrun/native-local-multinode-cell/local-multinode.log`
  - `target/spire-local-multinode-task121-native-dryrun/topology.local.json`
- no nested `bench-suite/*` artifacts are declared for this skipped bench run.

## Live Local Multinode Smoke

### Command

`target/debug/ecaz dev spire-multicluster local-multinode-pg18 --artifact-dir reviews/task-121/006-phase0-native-local-multinode-lane/artifacts/live-smoke2 --run-id task121-native-smoke2 --coord-port 39920 --remote1-port 39921 --remote2-port 39922 --remote3-port 39923 --tier correctness --storage-format rabitq --coord-index task121_native_coord_idx --remote-index task121_native_remote_idx --reloption nlists=128 --reloption recursive_fanout=8 --reloption top_graph_enabled=1 --skip-bench-suite --skip-fault-drills --skip-install`

### `live-smoke2/local-multinode.log`

- result: PASS
- key lines:
  - `coord_port=39920`
  - `remote1_port=39921`
  - `remote2_port=39922`
  - `remote3_port=39923`
  - `tier=correctness`
  - `prefix=ec_spire_aws_synth_10k`
  - `SPIRE local multinode fixture passed`
  - `HARNESS PASSED`

### `live-smoke2/coordinator-load.log`

- coordinator index: `task121_native_coord_idx`
- loaded generated synthetic 10k corpus and query table.
- key line: `Created index task121_native_coord_idx in 1.68s`

### `live-smoke2/distributed-correctness/distributed-placement-plan.json`

- total rows: `10000`
- remote row counts:
  - node 2: `3246`
  - node 3: `3388`
  - node 4: `3366`
- remote index regclass: `task121_native_remote_idx`
- storage format: `rabitq`

### Remote Load Logs

- `live-smoke2/remote-load-node-2.log`: loaded node 2 corpus and index.
- `live-smoke2/remote-load-node-3.log`: loaded node 3 corpus and index.
- `live-smoke2/remote-load-node-4.log`: loaded node 4 corpus and index.

### Remote Materialization Logs

- `live-smoke2/remote-leaf-materialization/node-2-remote-materialize.log`: `1 43 3246 139 9999 materialized`
- `live-smoke2/remote-leaf-materialization/node-3-remote-materialize.log`: `1 43 3388 139 10001 materialized`
- `live-smoke2/remote-leaf-materialization/node-4-remote-materialize.log`: `1 42 3366 139 9996 materialized`

### `live-smoke2/publish-remote-placements.log`

- key line: `1 128 3 published_static_remote_placements`

### `live-smoke2/register-remotes.log`

- registration succeeded for three remotes.
- notes: PostgreSQL emitted notices that `max_prepared_transactions=0`; those notices did not block registration.

### `live-smoke2/smoke-customscan-read.log`

- result: PASS
- registered remote node lines:
  - `0 active 10`
  - `2 active 43`
  - `3 active 43`
  - `4 active 42`
- distributed read evidence:
  - `Custom Scan (EcSpireDistributedScan)`
  - `remote_fanout: 3`
  - `tuple_transport_status: ready`

### `live-smoke2/production-read-profile-smoke.log`

- result: PASS
- key lines:
  - `status ready`
  - `result_source remote_heap_candidates`
  - `dispatch_count 3`
  - `remote_heap_ready_dispatch_count 3`
  - `returned_candidate_count 10`
  - `next_blocker none`

### `live-smoke2/bench-spire-pipeline-smoke.log`

- result: PASS
- endpoint tuple transport: `ready`
- default remote tuple transport: `pg_binary_attr_v1`
- production read profile: all statuses `ready`, result source `remote_heap_candidates`, no remote timeout/cancel/degraded skips.
- coordinator query metrics:
  - nprobe 8: 5 queries, p50 `40.325 ms`, recall@10 `0.2400`
  - nprobe 16: 5 queries, p50 `41.316 ms`, recall@10 `0.3200`
  - nprobe 32: 5 queries, p50 `41.478 ms`, recall@10 `0.5200`

## Excluded Local Outputs

The live run generated shard corpus TSVs and coordinator assignment TSVs under `live-smoke2/distributed-correctness/node-*`. Those files are not committed because review packet rules ban corpus/query/ground-truth TSVs under `reviews/`. The committed placement plan and logs above record the row counts and runtime evidence needed for review.
