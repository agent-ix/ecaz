# Task 144 Packet 008 Artifact Manifest

- head SHA: `a7857ae9f5f9459d3a9c4a442371fb5a97995c07`
- task bucket: `reviews/task-144`
- packet path: `reviews/task-144/008-release-matrix-config-r2`
- lane: local PG18 Task 144 release-matrix planning lane
- database: `tqvector_bench_task144`
- host / port: `/home/peter/dev/ecaz/target/task144-pg18-socket` / `28818`
- runner config: `artifacts/suite-task144-release-matrix-r2.json`
- timestamp: 2026-07-05

## Scope

This packet responds to review feedback on packets 006 and 007 before spending
50k/100k benchmark time.

Code commit `a7857ae9f` changes:

- `ecaz bench suite` now treats `recall.truth_cache_file` as an expected and
  produced artifact.
- Suite audit now models `recall.truth_corpus_file` and
  `spire-pipeline.truth_{corpus,cache}_file` as inputs, so a clean artifact
  directory must include a producer step before pipeline consumers.
- Suite result extraction now derives `spire_pipeline_row_scan` rows from
  `candidate_sum / queries / corpus_rows`, with ready-row percentages when
  `ready_sum` is present.
- `ecaz bench storage` now emits `storage_spire_replication` evidence for
  SPIRE indexes: `object_count`, `leaf_assignment_count`,
  `mean_replicas_per_vector`, `delta_assignment_count`, and health status.

## Config Shape

`suite-task144-release-matrix-r2.json` contains 124 steps:

- 1 release-profile precheck
- 15 load steps: 3 scales x 5 assignment variants
- 3 truth-cache recall steps, one per scale
- 15 storage steps
- 90 `spire-pipeline` steps: 3 scales x 5 assignment variants x 6 query modes

Assignment variants:

- `single`: `boundary_replica_count=0`, `closure_epsilon=0`
- `fixed_b2`: `boundary_replica_count=2`, `closure_epsilon=0`
- `closure_e010_b8`: `boundary_replica_count=8`, `closure_epsilon=0.10`
- `closure_e025_b8`: `boundary_replica_count=8`, `closure_epsilon=0.25`
- `closure_e050_b8`: `boundary_replica_count=8`, `closure_epsilon=0.50`

Query modes:

- fixed nprobe
- ratio pruning at `1.25`, `2.0`, `4.0`, `8.0`
- adaptive nprobe

The nprobe sweep remains `[8,16,32,64,96]`. This intentionally deviates from
the registered default `[8,16,24,32]` because packet 007 showed 10k only reaches
the 0.99 recall region around nprobe 32 and the ratio arms need a high fixed
ceiling to show whether the ratio band, not nprobe, is binding.

## Commands

Suite audit:

```text
target/debug/ecaz bench suite audit \
  --config reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json \
  > reviews/task-144/008-release-matrix-config-r2/artifacts/suite-audit.log 2>&1
```

Dry-run:

```text
target/debug/ecaz --database tqvector_bench_task144 \
  --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 \
  bench suite run --dry-run \
  --config reviews/task-144/008-release-matrix-config-r2/artifacts/suite-task144-release-matrix-r2.json \
  --artifact-dir reviews/task-144/008-release-matrix-config-r2/artifacts \
  --manifest-output reviews/task-144/008-release-matrix-config-r2/artifacts/dry-run-suite-manifest.json \
  --results-output reviews/task-144/008-release-matrix-config-r2/artifacts/dry-run-results.jsonl \
  > reviews/task-144/008-release-matrix-config-r2/artifacts/suite-dry-run.log 2>&1
```

Focused tests:

```text
cargo test -p ecaz-cli suite \
  > reviews/task-144/008-release-matrix-config-r2/artifacts/cargo-test-ecaz-cli-suite.log 2>&1

cargo test -p ecaz-cli storage \
  > reviews/task-144/008-release-matrix-config-r2/artifacts/cargo-test-ecaz-cli-storage.log 2>&1
```

## Key Results

- `suite-audit.log`: `[suite:task144-release-matrix-r2] audit passed: 124 steps`
- `dry-run-suite-manifest.json`: 124 dry-run steps, connection
  `tqvector_bench_task144` at `/home/peter/dev/ecaz/target/task144-pg18-socket:28818`
- `cargo-test-ecaz-cli-suite.log`: 62 passed
- `cargo-test-ecaz-cli-storage.log`: 13 passed

No corpus TSV, generated truth cache, or benchmark result cache is committed in
this packet.
