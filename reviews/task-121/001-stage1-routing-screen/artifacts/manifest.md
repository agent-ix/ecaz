# Task 121 Stage 1 Routing Screen Artifact Manifest

- Head SHA: `ab80fdc26a48bf9eebf49b15590e9d3d83ab661f`
- Task bucket: `reviews/task-121/001-stage1-routing-screen`
- Lane: `intel-local`
- Fixture: real corpus 100k, q20 bounded baseline slice plus q200/seven-sweep baseline
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

## q200 Baseline Pipeline

- `truth-cache-suite-manifest.json`
- `pipeline-baseline-incremental-suite-manifest.json`
- `pipeline-baseline-incremental-suite-results.jsonl`
- `pipeline-baseline-incremental-suite-run.log`
- `pipeline-baseline.log`
- `pipeline-baseline-funnel.jsonl`
- `pipeline-baseline-stage-containment.jsonl`
- `pipeline-baseline-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-baseline --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/pipeline-baseline-incremental-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/pipeline-baseline-incremental-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/pipeline-baseline-incremental-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.7250, p50 244.812 ms, p95 307.294 ms, p99 334.587 ms, max 345.912 ms
    - nprobe 16: recall@10 0.8525, p50 502.956 ms, p95 609.909 ms, p99 661.955 ms, max 788.417 ms
    - nprobe 24: recall@10 0.9045, p50 785.069 ms, p95 915.098 ms, p99 998.400 ms, max 1135.611 ms
    - nprobe 32: recall@10 0.9310, p50 1055.659 ms, p95 1214.091 ms, p99 1307.336 ms, max 1466.273 ms
    - nprobe 48: recall@10 0.9645, p50 1633.655 ms, p95 1836.622 ms, p99 2089.991 ms, max 2114.999 ms
    - nprobe 64: recall@10 0.9825, p50 2181.396 ms, p95 2481.401 ms, p99 2648.697 ms, max 2847.964 ms
    - nprobe 96: recall@10 0.9975, p50 3347.935 ms, p95 3676.094 ms, p99 4007.964 ms, max 4218.772 ms
  - Route-stage containment from `pipeline-baseline-route-containment.tsv`:
    - nprobe 8: 1450/2000 truth items contained, 0.7250
    - nprobe 16: 1705/2000 truth items contained, 0.8525
    - nprobe 24: 1809/2000 truth items contained, 0.9045
    - nprobe 32: 1862/2000 truth items contained, 0.9310
    - nprobe 48: 1929/2000 truth items contained, 0.9645
    - nprobe 64: 1965/2000 truth items contained, 0.9825
    - nprobe 96: 1995/2000 truth items contained, 0.9975
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,232,065, object_bytes_sum 1,004,577,112
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 2,514,557, object_bytes_sum 2,050,256,360
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 3,816,799, object_bytes_sum 3,112,033,720
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 5,165,224, object_bytes_sum 4,211,460,160
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 7,795,405, object_bytes_sum 6,355,962,304
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 10,420,357, object_bytes_sum 8,496,202,984
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 15,506,227, object_bytes_sum 12,642,962,128
  - Interpretation: route-stage containment exactly matches final recall at every nprobe, so this baseline is route-limited. Downstream stages did not introduce additional truth loss for the measured baseline.
  - Prior attempt: the original q200/seven-sweep pipeline was canceled after more than 33 minutes without visible intermediate result artifacts. The retry above used the incremental JSONL runner change from packet `003-spire-pipeline-incremental-jsonl` and completed with packet-local artifacts.
  - Note: q200 truth-cache JSON is a regenerable cache and is intentionally not committed.
