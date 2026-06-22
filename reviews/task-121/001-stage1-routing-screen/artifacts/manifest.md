# Task 121 Stage 1 Routing Screen Artifact Manifest

- Head SHA: `fedf1dbe609f1e6e2377c1ba84da5d4f460f6d3c`
- Task bucket: `reviews/task-121/001-stage1-routing-screen`
- Lane: `intel-local`
- Fixture: real corpus 100k, q20 bounded baseline slice
- Storage format: RaBitQ baseline
- Rerank mode: default SPIRE pipeline exact-source rerank
- Index/table isolation: isolated prefix/table/index for baseline (`t121_s1_100k_baseline`)
- Timestamp: 2026-06-22
- AWS: not used

## Config

- `suite-stage1-routing-screen-100k.json`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json`
  - Result: audit passed, 58 steps.
  - Notes: storage-format screen is RaBitQ baseline plus TurboQuant only. The suite adds q20 bounded baseline steps and `truth_cache_file` wiring for full q200 pipeline steps.

## Host Precheck

- `precheck-host.log`
  - Command: suite step `precheck-host`
  - Database/socket: `tqvector_bench_task121`, `/home/peter/.pgrx`, PG18 port `28818`
  - Key result: PG 18.3 local backend, release build profile, `ec_spire.nprobe=-1`, `ec_spire.rerank_width=-1`, `ec_spire.adaptive_nprobe=off`.

- `precheck-suite-manifest.json`
  - Command: `target/debug/ecaz bench suite run --config ... --only precheck-host --manifest-output ... --results-output ...`
  - Result: precheck completed.

## Baseline Load And Storage

- `load-baseline.log`
  - Command: suite step `load-baseline`
  - Key result lines:
    - copied corpus rows: 100000 in 95.55s
    - encoded corpus rows: 100000 in 36.24s
    - copied query rows: 1000 in 980.82ms
    - built index `t121_s1_100k_baseline_idx` in 11.60s
    - completed prefix in 310.09s

- `storage-baseline.log`
  - Command: suite step `storage-baseline`
  - Key result lines:
    - total: 1.6 GiB
    - SPIRE index: 79.7 MiB
    - index bytes/row: 835.8 B

- `baseline-suite-manifest.json`
  - Command: `target/debug/ecaz bench suite run --config ... --only-tag lever=baseline --resume-from reviews/task-121/001-stage1-routing-screen/artifacts/precheck-suite-manifest.json --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/baseline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/baseline-suite-results.jsonl`
  - Result: load/storage completed; later q200 pipeline canceled before producing results.

## Bounded q20 Truth Cache And Recall

- `truth-cache-q20-suite-manifest.json`
- `truth-cache-q20-suite-results.jsonl`
- `truth-cache-100k-q20-k10.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only truth-cache-100k-q20-k10 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/truth-cache-q20-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/truth-cache-q20-suite-results.jsonl`
  - Key result lines:
    - computed ground truth: 20 queries vs 100000 corpus rows, dim 1536
    - ground truth in 7.74s
    - nprobe 96, queries 20, recall@10 1.0000, CI95 low 0.9812, ndcg@10 1.0000, mean q-time 3363.50 ms
  - Note: `truth-cache-100k-q20-k10.json` is a regenerable cache and is intentionally not committed.

## Bounded q20 Baseline Pipeline

- `pipeline-baseline-q20-n96-suite-manifest.json`
- `pipeline-baseline-q20-n96-suite-results.jsonl`
- `pipeline-baseline-q20-n96.log`
- `pipeline-baseline-q20-n96-funnel.jsonl`
- `pipeline-baseline-q20-n96-stage-containment.jsonl`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-baseline-q20-n96 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/pipeline-baseline-q20-n96-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/pipeline-baseline-q20-n96-suite-results.jsonl`
  - Key result lines:
    - tuple transport status: ready, default `pg_binary_attr_v1`
    - nprobe 96, queries 20, recall@10 1.0000
    - latency min 3078.257 ms, p50 3238.948 ms, p95 3411.542 ms, p99 3909.826 ms, max 3909.826 ms
    - routing status `truncated`, route_sum 1920, `next_blocker=routing_budget`
    - candidates ready: 1,522,002
    - heap rerank rows: 1,522,002
    - local store object bytes: 1,240,964,832

## q200 Attempt

- `pipeline-baseline-suite-manifest.json`
- `truth-cache-suite-manifest.json`
  - Command: q200 truth-cache generation followed by q200/seven-sweep `pipeline-baseline`.
  - Useful result: q200 truth cache generated in 28.69s before the redundant recall step was canceled.
  - Negative result: q200/seven-sweep pipeline loaded cached truth but remained in `include_query_metrics` KNN queries for more than 33 minutes without writing result artifacts; canceled with `pg_cancel_backend`.
  - Note: q200 truth-cache JSON is a regenerable cache and is intentionally not committed.
