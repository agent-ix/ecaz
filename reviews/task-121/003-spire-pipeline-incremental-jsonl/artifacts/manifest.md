# Task 121 Packet 003 Artifact Manifest

- Head SHA: `64d7d1f843269434de5f651452a3c79a26518290`
- Task bucket: `reviews/task-121/003-spire-pipeline-incremental-jsonl`
- Lane: `intel-local`
- Fixture: Task 121 100k baseline, bounded q20/nprobe96 smoke
- Storage format: RaBitQ
- Index/table isolation: existing isolated baseline prefix/table/index
  `t121_s1_100k_baseline`
- Timestamp: 2026-06-22
- AWS: not used

## Code Validation

- `cargo-test-spire-pipeline.log`
  - Command: `cargo test -p ecaz-cli commands::bench::spire_pipeline`
  - Result: 19 passed, 0 failed.

- `cargo-build-ecaz-cli.log`
  - Command: `cargo build -p ecaz-cli --bin ecaz`
  - Result: passed.
  - Warning: pre-existing `LoadedDistributedPlacementConfig.path` dead-code warning.

## q20 Truth Cache Setup

- `truth-cache-q20-suite-run.log`
- `truth-cache-q20-suite-manifest.json`
- `truth-cache-q20-suite-results.jsonl`
- `truth-cache-100k-q20-k10.log`
  - Command:
    `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --artifact-dir reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only truth-cache-100k-q20-k10 --manifest-output reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts/truth-cache-q20-suite-manifest.json --results-output reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts/truth-cache-q20-suite-results.jsonl --log-file reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts/truth-cache-q20-suite-run.log`
  - Key result lines:
    - computed ground truth: 20 queries vs 100000 corpus rows, dim 1536
    - ground truth in 7.78s
    - nprobe 96, queries 20, recall@10 1.0000, CI95 low 0.9812, mean q-time 3270.57 ms
  - Note: `truth-cache-100k-q20-k10.json` is regenerable and intentionally not committed.

## Incremental JSONL Live Check

- `pipeline-baseline-q20-n96-suite-run.log`
- `pipeline-baseline-q20-n96-suite-manifest.json`
- `pipeline-baseline-q20-n96-suite-results.jsonl`
- `pipeline-baseline-q20-n96.log`
- `pipeline-baseline-q20-n96-funnel.jsonl`
- `pipeline-baseline-q20-n96-stage-containment.jsonl`
  - Command:
    `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --artifact-dir reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-baseline-q20-n96 --manifest-output reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts/pipeline-baseline-q20-n96-suite-manifest.json --results-output reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts/pipeline-baseline-q20-n96-suite-results.jsonl --log-file reviews/task-121/003-spire-pipeline-incremental-jsonl/artifacts/pipeline-baseline-q20-n96-suite-run.log`
  - Mid-run observation while the backend was still active:
    - `pipeline-baseline-q20-n96-funnel.jsonl`: 29,461 bytes
    - `pipeline-baseline-q20-n96-stage-containment.jsonl`: 50,536 bytes
  - Final result lines:
    - tuple transport status: ready, default `pg_binary_attr_v1`
    - nprobe 96, queries 20, recall@10 1.0000
    - latency min 3084.258 ms, p50 3248.748 ms, p95 3316.355 ms, p99/max 3844.312 ms
    - routing status `truncated`, `next_blocker=routing_budget`
    - candidates ready: 1,522,002
    - heap rerank rows: 1,522,002
