# Task 121 Stage 1 Routing Screen Artifact Manifest

- Head SHA: `353501957dfa1bf53e94878fd3e7ac014a0bfe97`
- Task bucket: `reviews/task-121/001-stage1-routing-screen`
- Lane: `intel-local`
- Fixture: real corpus 100k, q20 bounded baseline slice, q200/seven-sweep baseline, and q200 OFAT screens
- Storage format: RaBitQ baseline
- Rerank mode: default SPIRE pipeline exact-source rerank
- Index/table isolation: isolated prefix/table/index for baseline (`t121_s1_100k_baseline`)
- Timestamp: 2026-06-22
- AWS: not used

## Config

- `suite-stage1-routing-screen-100k.json`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json`
  - Result: audit passed, 59 steps.
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

## top_graph_search_list_size=200 OFAT

- `tgsl200-load-suite-manifest.json`
- `tgsl200-load-suite-results.jsonl`
- `tgsl200-load-suite-run.log`
- `load-tgsl200.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-tgsl200 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 96.69s
    - encoded corpus rows: 100000 in 33.65s
    - copied query rows: 1000 in 973.29ms
    - built index `t121_s1_100k_tgsl200_idx` in 9.39s
    - completed prefix in 296.20s

- `tgsl200-storage-suite-manifest.json`
- `tgsl200-storage-suite-results.jsonl`
- `tgsl200-storage-suite-run.log`
- `storage-tgsl200.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-tgsl200 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-storage-suite-run.log`
  - Key result lines:
    - total: 1.6 GiB
    - SPIRE index: 79.7 MiB
    - index bytes/row: 835.8 B

- `tgsl200-pipeline-suite-manifest.json`
- `tgsl200-pipeline-suite-results.jsonl`
- `tgsl200-pipeline-suite-run.log`
- `pipeline-tgsl200.log`
- `pipeline-tgsl200-funnel.jsonl`
- `pipeline-tgsl200-stage-containment.jsonl`
- `pipeline-tgsl200-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-tgsl200 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.7250, p50 257.170 ms, p95 327.460 ms, p99 373.368 ms, max 392.591 ms
    - nprobe 16: recall@10 0.8525, p50 518.332 ms, p95 622.913 ms, p99 677.147 ms, max 734.309 ms
    - nprobe 24: recall@10 0.9045, p50 816.473 ms, p95 947.148 ms, p99 1141.587 ms, max 1241.734 ms
    - nprobe 32: recall@10 0.9310, p50 1081.118 ms, p95 1256.546 ms, p99 1502.441 ms, max 1575.107 ms
    - nprobe 48: recall@10 0.9645, p50 1680.941 ms, p95 1903.131 ms, p99 2257.859 ms, max 2351.692 ms
    - nprobe 64: recall@10 0.9825, p50 2207.241 ms, p95 2482.774 ms, p99 2911.217 ms, max 3112.201 ms
    - nprobe 96: recall@10 0.9975, p50 3308.038 ms, p95 3749.160 ms, p99 4105.172 ms, max 4198.029 ms
  - Route-stage containment from `pipeline-tgsl200-route-containment.tsv` exactly matched baseline:
    - nprobe 8: 1450/2000 truth items contained, 0.7250
    - nprobe 16: 1705/2000 truth items contained, 0.8525
    - nprobe 24: 1809/2000 truth items contained, 0.9045
    - nprobe 32: 1862/2000 truth items contained, 0.9310
    - nprobe 48: 1929/2000 truth items contained, 0.9645
    - nprobe 64: 1965/2000 truth items contained, 0.9825
    - nprobe 96: 1995/2000 truth items contained, 0.9975
  - Pipeline counters matched baseline exactly at every nprobe, including candidate_sum and object_bytes_sum.
  - Interpretation: the low-nprobe `top_graph_search_list_size=200` sweep did not improve route containment or recall over the baseline `top_graph_search_list_size=96`, but it is inert for beam analysis because every measured nprobe was <= 96. Treat it as a baseline-equivalence/control result, not as a final beam verdict.

- `tgsl200-hi-pipeline-suite-manifest.json`
- `tgsl200-hi-pipeline-suite-results.jsonl`
- `tgsl200-hi-pipeline-suite-run.log`
- `pipeline-tgsl200-hi.log`
- `pipeline-tgsl200-hi-funnel.jsonl`
- `pipeline-tgsl200-hi-stage-containment.jsonl`
- `pipeline-tgsl200-hi-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-tgsl200-hi --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-hi-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-hi-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/tgsl200-hi-pipeline-suite-run.log`
  - Results shape: 200 queries x three nprobe values (`128,160,200`), 600 funnel rows, 3,600 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 128: recall@10 1.0000, p50 4351.972 ms, p95 5125.647 ms, p99 5424.095 ms, max 8445.497 ms
    - nprobe 160: recall@10 1.0000, p50 4364.396 ms, p95 5296.145 ms, p99 5478.484 ms, max 5553.372 ms
    - nprobe 200: recall@10 1.0000, p50 4410.790 ms, p95 4967.462 ms, p99 5100.958 ms, max 5313.000 ms
  - Route-stage containment from `pipeline-tgsl200-hi-route-containment.tsv`:
    - nprobe 128: 2000/2000 truth items contained, 1.0000
    - nprobe 160: 2000/2000 truth items contained, 1.0000
    - nprobe 200: 2000/2000 truth items contained, 1.0000
  - Pipeline counters:
    - nprobe 128: effective_nprobe 128, beam_width 128, max_leaf_routes 128, truncation `none`, candidate_sum 20,000,000, heap_rerank_sum 20,000,000, object_bytes_sum 16,307,043,200
    - nprobe 160: effective_nprobe 128, beam_width 128, max_leaf_routes 128, truncation `none`, candidate_sum 20,000,000, heap_rerank_sum 20,000,000, object_bytes_sum 16,307,043,200
    - nprobe 200: effective_nprobe 128, beam_width 128, max_leaf_routes 128, truncation `none`, candidate_sum 20,000,000, heap_rerank_sum 20,000,000, object_bytes_sum 16,307,043,200
  - Interpretation: the corrected high-nprobe `top_graph_search_list_size=200` run reaches perfect route containment and recall, but only by materializing and heap-reranking the full 100k corpus for every query. It is a useful ceiling/diagnostic result, not a practical performance lever for this 100k local screen without another pruning change.

## boundary_replica_count=1 OFAT

- `bound1-load-suite-manifest.json`
- `bound1-load-suite-results.jsonl`
- `bound1-load-suite-run.log`
- `load-bound1.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-bound1 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound1-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound1-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound1-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 98.37s
    - encoded corpus rows: 100000 in 32.18s
    - copied query rows: 1000 in 1.02s
    - built index `t121_s1_100k_bound1_idx` in 44.40s
    - completed prefix in 333.24s

- `bound1-storage-suite-manifest.json`
- `bound1-storage-suite-results.jsonl`
- `bound1-storage-suite-run.log`
- `storage-bound1.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-bound1 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound1-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound1-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound1-storage-suite-run.log`
  - Key result lines:
    - total: 1.7 GiB
    - indexes: 160.1 MiB
    - SPIRE index: 157.9 MiB
    - index bytes/row: 1655.2 B

- `bound1-pipeline-suite-manifest.json`
- `bound1-pipeline-suite-results.jsonl`
- `bound1-pipeline-suite-run.log`
- `pipeline-bound1.log`
- `pipeline-bound1-funnel.jsonl`
- `pipeline-bound1-stage-containment.jsonl`
- `pipeline-bound1-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-bound1 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound1-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound1-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound1-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.8365, p50 497.014 ms, p95 634.028 ms, p99 820.927 ms, max 1031.618 ms
    - nprobe 16: recall@10 0.9235, p50 950.199 ms, p95 1167.817 ms, p99 1292.569 ms, max 1489.419 ms
    - nprobe 24: recall@10 0.9605, p50 1335.556 ms, p95 1585.095 ms, p99 1719.755 ms, max 1940.050 ms
    - nprobe 32: recall@10 0.9735, p50 1742.360 ms, p95 2020.705 ms, p99 2175.389 ms, max 2718.170 ms
    - nprobe 48: recall@10 0.9870, p50 2449.828 ms, p95 2873.829 ms, p99 3037.819 ms, max 3138.740 ms
    - nprobe 64: recall@10 0.9940, p50 3146.297 ms, p95 3592.560 ms, p99 4101.957 ms, max 4437.518 ms
    - nprobe 96: recall@10 0.9995, p50 4137.887 ms, p95 4579.126 ms, p99 4977.837 ms, max 5257.249 ms
  - Route-stage containment from `pipeline-bound1-route-containment.tsv`:
    - nprobe 8: 1673/2000 truth items contained, 0.8365
    - nprobe 16: 1847/2000 truth items contained, 0.9235
    - nprobe 24: 1921/2000 truth items contained, 0.9605
    - nprobe 32: 1947/2000 truth items contained, 0.9735
    - nprobe 48: 1974/2000 truth items contained, 0.9870
    - nprobe 64: 1988/2000 truth items contained, 0.9940
    - nprobe 96: 1999/2000 truth items contained, 0.9995
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 2,638,655, object_bytes_sum 2,151,211,640
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 5,243,699, object_bytes_sum 4,275,026,696
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 7,848,831, object_bytes_sum 6,398,915,592
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 10,487,717, object_bytes_sum 8,550,319,352
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 15,763,335, object_bytes_sum 12,851,374,848
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 21,025,889, object_bytes_sum 17,141,781,392
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 31,137,800, object_bytes_sum 25,385,760,368
  - Interpretation: `boundary_replica_count=1` substantially improves route containment and final recall at every nprobe, but roughly doubles candidate work and SPIRE index size compared with the baseline. Route-stage containment still exactly matches final recall, so the measured pipeline remains route-limited.

## boundary_replica_count=2 OFAT

- `bound2-load-suite-manifest.json`
- `bound2-load-suite-results.jsonl`
- `bound2-load-suite-run.log`
- `load-bound2.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-bound2 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound2-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound2-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound2-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 98.71s
    - encoded corpus rows: 100000 in 39.58s
    - copied query rows: 1000 in 1.05s
    - built index `t121_s1_100k_bound2_idx` in 54.47s
    - completed prefix in 360.75s

- `bound2-storage-suite-manifest.json`
- `bound2-storage-suite-results.jsonl`
- `bound2-storage-suite-run.log`
- `storage-bound2.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-bound2 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound2-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound2-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound2-storage-suite-run.log`
  - Key result lines:
    - total: 1.8 GiB
    - indexes: 238.2 MiB
    - SPIRE index: 235.9 MiB
    - index bytes/row: 2473.8 B

- `bound2-pipeline-suite-manifest.json`
- `bound2-pipeline-suite-results.jsonl`
- `bound2-pipeline-suite-run.log`
- `pipeline-bound2.log`
- `pipeline-bound2-funnel.jsonl`
- `pipeline-bound2-stage-containment.jsonl`
- `pipeline-bound2-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-bound2 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound2-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound2-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound2-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.8730, p50 696.395 ms, p95 894.634 ms, p99 1032.729 ms, max 1234.470 ms
    - nprobe 16: recall@10 0.9465, p50 1227.200 ms, p95 1565.707 ms, p99 1817.870 ms, max 2015.246 ms
    - nprobe 24: recall@10 0.9745, p50 1737.343 ms, p95 2160.530 ms, p99 2362.026 ms, max 2669.731 ms
    - nprobe 32: recall@10 0.9835, p50 2181.439 ms, p95 2686.965 ms, p99 2949.596 ms, max 3026.499 ms
    - nprobe 48: recall@10 0.9925, p50 3034.067 ms, p95 3587.567 ms, p99 4017.705 ms, max 4175.763 ms
    - nprobe 64: recall@10 0.9970, p50 3607.115 ms, p95 4125.617 ms, p99 4343.296 ms, max 4843.655 ms
    - nprobe 96: recall@10 1.0000, p50 4553.530 ms, p95 5164.317 ms, p99 5450.587 ms, max 5589.324 ms
  - Route-stage containment from `pipeline-bound2-route-containment.tsv`:
    - nprobe 8: 1746/2000 truth items contained, 0.8730
    - nprobe 16: 1893/2000 truth items contained, 0.9465
    - nprobe 24: 1949/2000 truth items contained, 0.9745
    - nprobe 32: 1967/2000 truth items contained, 0.9835
    - nprobe 48: 1985/2000 truth items contained, 0.9925
    - nprobe 64: 1994/2000 truth items contained, 0.9970
    - nprobe 96: 2000/2000 truth items contained, 1.0000
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 3,942,723, object_bytes_sum 3,214,292,280
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 7,878,446, object_bytes_sum 6,422,875,136
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 11,786,881, object_bytes_sum 9,609,210,352
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 15,730,984, object_bytes_sum 12,824,622,496
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 23,657,267, object_bytes_sum 19,286,491,136
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 31,551,045, object_bytes_sum 25,721,865,624
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 46,704,408, object_bytes_sum 38,075,629,848
  - Interpretation: `boundary_replica_count=2` continues improving route containment and final recall, reaching 1.0000 recall@10 at nprobe 96, but with diminishing incremental gains over `boundary_replica_count=1` and a steep cost increase. Route-stage containment still exactly matches final recall, so downstream stages remain lossless for this measured local pipeline.

## boundary_replica_count=4 OFAT

- `bound4-load-suite-manifest.json`
- `bound4-load-suite-results.jsonl`
- `bound4-load-suite-run.log`
- `load-bound4.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-bound4 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound4-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound4-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound4-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 105.23s
    - encoded corpus rows: 100000 in 40.66s
    - copied query rows: 1000 in 1.13s
    - built index `t121_s1_100k_bound4_idx` in 61.94s
    - completed prefix in 377.89s

- `bound4-storage-suite-manifest.json`
- `bound4-storage-suite-results.jsonl`
- `bound4-storage-suite-run.log`
- `storage-bound4.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-bound4 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound4-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound4-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound4-storage-suite-run.log`
  - Key result lines:
    - total: 1.9 GiB
    - indexes: 394.5 MiB
    - SPIRE index: 392.2 MiB
    - index bytes/row: 4112.4 B

- `bound4-pipeline-suite-manifest.json`
- `bound4-pipeline-suite-results.jsonl`
- `bound4-pipeline-suite-run.log`
- `pipeline-bound4.log`
- `pipeline-bound4-funnel.jsonl`
- `pipeline-bound4-stage-containment.jsonl`
- `pipeline-bound4-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-bound4 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/bound4-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/bound4-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/bound4-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.9135, p50 1017.592 ms, p95 1340.120 ms, p99 1445.754 ms, max 1795.534 ms
    - nprobe 16: recall@10 0.9655, p50 1791.907 ms, p95 2216.811 ms, p99 2434.887 ms, max 2483.666 ms
    - nprobe 24: recall@10 0.9875, p50 2368.554 ms, p95 2880.358 ms, p99 3090.746 ms, max 3359.237 ms
    - nprobe 32: recall@10 0.9915, p50 2899.310 ms, p95 3537.148 ms, p99 3950.738 ms, max 4116.895 ms
    - nprobe 48: recall@10 0.9955, p50 3695.809 ms, p95 4414.522 ms, p99 4679.280 ms, max 5138.083 ms
    - nprobe 64: recall@10 0.9985, p50 4242.798 ms, p95 5001.764 ms, p99 5388.172 ms, max 5471.008 ms
    - nprobe 96: recall@10 1.0000, p50 4948.452 ms, p95 5773.607 ms, p99 6118.705 ms, max 6313.436 ms
  - Route-stage containment from `pipeline-bound4-route-containment.tsv`:
    - nprobe 8: 1827/2000 truth items contained, 0.9135
    - nprobe 16: 1931/2000 truth items contained, 0.9655
    - nprobe 24: 1975/2000 truth items contained, 0.9875
    - nprobe 32: 1983/2000 truth items contained, 0.9915
    - nprobe 48: 1991/2000 truth items contained, 0.9955
    - nprobe 64: 1997/2000 truth items contained, 0.9985
    - nprobe 96: 2000/2000 truth items contained, 1.0000
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 6,473,923, object_bytes_sum 5,277,738,976
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 13,094,002, object_bytes_sum 10,674,617,080
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 19,636,910, object_bytes_sum 16,008,581,936
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 26,182,573, object_bytes_sum 21,344,792,056
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 39,338,713, object_bytes_sum 32,070,047,488
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 52,438,585, object_bytes_sum 42,749,433,016
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 77,582,377, object_bytes_sum 63,247,390,288
  - Interpretation: `boundary_replica_count=4` improves low-nprobe route containment again, but the incremental gain over `boundary_replica_count=2` is already small at medium/high nprobe and zero at nprobe 96. It is not a practical default candidate for this 100k local screen because storage and candidate fanout rise sharply; it is useful as the upper point in the reviewer-requested 1/2/4 knee sweep.

## nlists=316 OFAT

- `nlist316-load-suite-manifest.json`
- `nlist316-load-suite-results.jsonl`
- `nlist316-load-suite-run.log`
- `load-nlist316.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-nlist316 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 99.38s
    - encoded corpus rows: 100000 in 41.45s
    - copied query rows: 1000 in 1.01s
    - built index `t121_s1_100k_nlist316_idx` in 19.93s
    - completed prefix in 330.20s

- `nlist316-storage-suite-manifest.json`
- `nlist316-storage-suite-results.jsonl`
- `nlist316-storage-suite-run.log`
- `storage-nlist316.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-nlist316 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-storage-suite-run.log`
  - Key result lines:
    - total: 1.6 GiB
    - indexes: 84.0 MiB
    - SPIRE index: 81.8 MiB
    - index bytes/row: 858.1 B

- `nlist316-pipeline-suite-manifest.json`
- `nlist316-pipeline-suite-results.jsonl`
- `nlist316-pipeline-suite-run.log`
- `pipeline-nlist316.log`
- `pipeline-nlist316-funnel.jsonl`
- `pipeline-nlist316-stage-containment.jsonl`
- `pipeline-nlist316-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-nlist316 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist316-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.7370, p50 110.945 ms, p95 160.774 ms, p99 189.428 ms, max 215.405 ms
    - nprobe 16: recall@10 0.8230, p50 207.869 ms, p95 263.543 ms, p99 292.891 ms, max 322.823 ms
    - nprobe 24: recall@10 0.8650, p50 321.769 ms, p95 394.223 ms, p99 479.944 ms, max 517.561 ms
    - nprobe 32: recall@10 0.8975, p50 435.530 ms, p95 531.639 ms, p99 668.909 ms, max 708.045 ms
    - nprobe 48: recall@10 0.9220, p50 654.295 ms, p95 779.757 ms, p99 888.993 ms, max 927.761 ms
    - nprobe 64: recall@10 0.9445, p50 876.194 ms, p95 1047.150 ms, p99 1212.010 ms, max 1348.545 ms
    - nprobe 96: recall@10 0.9720, p50 1351.068 ms, p95 1568.207 ms, p99 1810.746 ms, max 1868.750 ms
  - Route-stage containment from `pipeline-nlist316-route-containment.tsv`:
    - nprobe 8: 1474/2000 truth items contained, 0.7370
    - nprobe 16: 1646/2000 truth items contained, 0.8230
    - nprobe 24: 1730/2000 truth items contained, 0.8650
    - nprobe 32: 1795/2000 truth items contained, 0.8975
    - nprobe 48: 1844/2000 truth items contained, 0.9220
    - nprobe 64: 1889/2000 truth items contained, 0.9445
    - nprobe 96: 1944/2000 truth items contained, 0.9720
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 487,390, object_bytes_sum 397,501,240
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 999,460, object_bytes_sum 815,123,560
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,500,025, object_bytes_sum 1,223,368,264
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 2,009,214, object_bytes_sum 1,638,645,528
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 3,051,404, object_bytes_sum 2,488,608,128
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 4,062,261, object_bytes_sum 3,313,029,384
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 6,137,532, object_bytes_sum 5,005,531,872
  - Interpretation: `nlists=316` reduces candidate fanout, object bytes, and latency substantially, with SPIRE index size close to baseline, but it does not fix route-stage loss. Route-stage containment exactly matches final recall at every nprobe, and recall is below baseline from nprobe 16 through nprobe 96. Treat this as a scan-efficiency/storage lever that may be useful in combination with a coverage lever, not as a standalone significant route-recall lever.

## nlists=512 OFAT

- `nlist512-load-suite-manifest.json`
- `nlist512-load-suite-results.jsonl`
- `nlist512-load-suite-run.log`
- `load-nlist512.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-nlist512 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 100.96s
    - encoded corpus rows: 100000 in 39.90s
    - copied query rows: 1000 in 1.04s
    - built index `t121_s1_100k_nlist512_idx` in 17.54s
    - completed prefix in 318.43s

- `nlist512-storage-suite-manifest.json`
- `nlist512-storage-suite-results.jsonl`
- `nlist512-storage-suite-run.log`
- `storage-nlist512.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-nlist512 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-storage-suite-run.log`
  - Key result lines:
    - total: 1.6 GiB
    - indexes: 86.2 MiB
    - SPIRE index: 84.0 MiB
    - index bytes/row: 881.1 B

- `nlist512-pipeline-suite-manifest.json`
- `nlist512-pipeline-suite-results.jsonl`
- `nlist512-pipeline-suite-run.log`
- `pipeline-nlist512.log`
- `pipeline-nlist512-funnel.jsonl`
- `pipeline-nlist512-stage-containment.jsonl`
- `pipeline-nlist512-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-nlist512 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist512-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.6600, p50 82.284 ms, p95 127.963 ms, p99 160.771 ms, max 187.125 ms
    - nprobe 16: recall@10 0.7630, p50 145.977 ms, p95 191.681 ms, p99 208.049 ms, max 221.874 ms
    - nprobe 24: recall@10 0.8125, p50 214.549 ms, p95 269.854 ms, p99 306.690 ms, max 409.914 ms
    - nprobe 32: recall@10 0.8480, p50 289.288 ms, p95 371.181 ms, p99 428.569 ms, max 468.653 ms
    - nprobe 48: recall@10 0.8850, p50 437.427 ms, p95 526.012 ms, p99 554.278 ms, max 714.449 ms
    - nprobe 64: recall@10 0.9075, p50 590.094 ms, p95 721.693 ms, p99 759.129 ms, max 856.643 ms
    - nprobe 96: recall@10 0.9350, p50 890.681 ms, p95 1156.328 ms, p99 1296.603 ms, max 1609.012 ms
  - Route-stage containment from `pipeline-nlist512-route-containment.tsv`:
    - nprobe 8: 1320/2000 truth items contained, 0.6600
    - nprobe 16: 1526/2000 truth items contained, 0.7630
    - nprobe 24: 1625/2000 truth items contained, 0.8125
    - nprobe 32: 1696/2000 truth items contained, 0.8480
    - nprobe 48: 1770/2000 truth items contained, 0.8850
    - nprobe 64: 1815/2000 truth items contained, 0.9075
    - nprobe 96: 1870/2000 truth items contained, 0.9350
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 329,783, object_bytes_sum 269,022,296
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 652,296, object_bytes_sum 532,121,472
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 983,356, object_bytes_sum 802,188,544
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,319,095, object_bytes_sum 1,076,069,656
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,984,266, object_bytes_sum 1,618,690,104
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 2,654,041, object_bytes_sum 2,165,062,552
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 3,991,647, object_bytes_sum 3,256,222,152
  - Interpretation: `nlists=512` further reduces fanout and latency relative to `nlists=316`, but worsens route-stage containment at every nprobe. The result reinforces that higher `nlists` alone is a scan-efficiency/storage lever under the fixed routing budget, not a route-recall recovery lever.

## nlists=1024 OFAT

- `nlist1024-load-suite-manifest.json`
- `nlist1024-load-suite-results.jsonl`
- `nlist1024-load-suite-run.log`
- `load-nlist1024.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-nlist1024 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 101.78s
    - encoded corpus rows: 100000 in 38.29s
    - copied query rows: 1000 in 1.03s
    - built index `t121_s1_100k_nlist1024_idx` in 24.97s
    - completed prefix in 327.87s

- `nlist1024-storage-suite-manifest.json`
- `nlist1024-storage-suite-results.jsonl`
- `nlist1024-storage-suite-run.log`
- `storage-nlist1024.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-nlist1024 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-storage-suite-run.log`
  - Key result lines:
    - total: 1.6 GiB
    - indexes: 92.0 MiB
    - SPIRE index: 89.8 MiB
    - index bytes/row: 941.2 B

- `nlist1024-pipeline-suite-manifest.json`
- `nlist1024-pipeline-suite-results.jsonl`
- `nlist1024-pipeline-suite-run.log`
- `pipeline-nlist1024.log`
- `pipeline-nlist1024-funnel.jsonl`
- `pipeline-nlist1024-stage-containment.jsonl`
- `pipeline-nlist1024-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-nlist1024 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/nlist1024-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.6410, p50 83.488 ms, p95 114.339 ms, p99 127.886 ms, max 136.624 ms
    - nprobe 16: recall@10 0.7190, p50 118.638 ms, p95 153.161 ms, p99 173.645 ms, max 180.503 ms
    - nprobe 24: recall@10 0.7725, p50 161.101 ms, p95 200.097 ms, p99 250.597 ms, max 271.928 ms
    - nprobe 32: recall@10 0.8110, p50 191.812 ms, p95 235.784 ms, p99 301.654 ms, max 321.914 ms
    - nprobe 48: recall@10 0.8635, p50 272.876 ms, p95 340.202 ms, p99 370.248 ms, max 633.631 ms
    - nprobe 64: recall@10 0.8910, p50 364.600 ms, p95 456.640 ms, p99 487.897 ms, max 551.777 ms
    - nprobe 96: recall@10 0.9215, p50 517.387 ms, p95 647.998 ms, p99 704.917 ms, max 852.538 ms
  - Route-stage containment from `pipeline-nlist1024-route-containment.tsv`:
    - nprobe 8: 1282/2000 truth items contained, 0.6410
    - nprobe 16: 1438/2000 truth items contained, 0.7190
    - nprobe 24: 1545/2000 truth items contained, 0.7725
    - nprobe 32: 1622/2000 truth items contained, 0.8110
    - nprobe 48: 1727/2000 truth items contained, 0.8635
    - nprobe 64: 1782/2000 truth items contained, 0.8910
    - nprobe 96: 1843/2000 truth items contained, 0.9215
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 196,031, object_bytes_sum 159,999,200
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 390,113, object_bytes_sum 318,402,728
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 575,635, object_bytes_sum 469,828,576
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 756,686, object_bytes_sum 617,608,736
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,113,024, object_bytes_sum 908,469,624
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,474,921, object_bytes_sum 1,203,862,648
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 2,177,568, object_bytes_sum 1,777,407,696
  - Interpretation: `nlists=1024` completes the standalone `nlists` boundary check. It reduces fanout and latency again, but route-stage containment and final recall continue to degrade under the fixed routing budget. Treat higher `nlists` as a cost lever that may need a coverage lever paired with it, not as a standalone route-recall lever.

## recursive_fanout=0 Build-Gated Cell

- `fanout0-load-suite-manifest.json`
- `fanout0-load-suite-run.log`
- `load-fanout0.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-fanout0 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout0-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout0-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/fanout0-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 101.26s
    - encoded corpus rows: 100000 in 32.20s
    - copied query rows: 1000 in 1.01s
    - index build failed with `ERROR: ec_spire top_graph_enabled requires recursive_fanout >= 2 during build`
  - Interpretation: the literal `recursive_fanout=0` OFAT cell is invalid with the Task 121 baseline because `top_graph_enabled=1` is held constant. No route-containment or recall result exists for this build-gated combination.

## recursive_fanout=16 OFAT

- `fanout16-load-suite-manifest.json`
- `fanout16-load-suite-results.jsonl`
- `fanout16-load-suite-run.log`
- `load-fanout16.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only load-fanout16 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-load-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-load-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-load-suite-run.log`
  - Key result lines:
    - copied corpus rows: 100000 in 97.85s
    - encoded corpus rows: 100000 in 33.15s
    - copied query rows: 1000 in 1.01s
    - built index `t121_s1_100k_fanout16_idx` in 10.10s
    - completed prefix in 301.75s

- `fanout16-storage-suite-manifest.json`
- `fanout16-storage-suite-results.jsonl`
- `fanout16-storage-suite-run.log`
- `storage-fanout16.log`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only storage-fanout16 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-storage-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-storage-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-storage-suite-run.log`
  - Key result lines:
    - total: 1.6 GiB
    - indexes: 82.0 MiB
    - SPIRE index: 79.8 MiB
    - index bytes/row: 836.6 B

- `fanout16-pipeline-suite-manifest.json`
- `fanout16-pipeline-suite-results.jsonl`
- `fanout16-pipeline-suite-run.log`
- `pipeline-fanout16.log`
- `pipeline-fanout16-funnel.jsonl`
- `pipeline-fanout16-stage-containment.jsonl`
- `pipeline-fanout16-route-containment.tsv`
  - Command: `target/debug/ecaz bench suite run --config reviews/task-121/001-stage1-routing-screen/artifacts/suite-stage1-routing-screen-100k.json --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 --only pipeline-fanout16 --manifest-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-pipeline-suite-manifest.json --results-output reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-pipeline-suite-results.jsonl --log-file reviews/task-121/001-stage1-routing-screen/artifacts/fanout16-pipeline-suite-run.log`
  - Results shape: 200 queries x seven nprobe values (`8,16,24,32,48,64,96`), 1,400 funnel rows, 8,400 stage-containment rows.
  - Key coordinator result lines:
    - nprobe 8: recall@10 0.7480, p50 250.995 ms, p95 324.399 ms, p99 343.845 ms, max 361.317 ms
    - nprobe 16: recall@10 0.8605, p50 534.268 ms, p95 628.510 ms, p99 662.994 ms, max 684.571 ms
    - nprobe 24: recall@10 0.9170, p50 797.713 ms, p95 886.185 ms, p99 924.432 ms, max 930.390 ms
    - nprobe 32: recall@10 0.9400, p50 1073.706 ms, p95 1191.517 ms, p99 1241.229 ms, max 1409.611 ms
    - nprobe 48: recall@10 0.9680, p50 1626.846 ms, p95 1783.439 ms, p99 1827.741 ms, max 1851.927 ms
    - nprobe 64: recall@10 0.9845, p50 2187.801 ms, p95 2340.580 ms, p99 2458.388 ms, max 2739.102 ms
    - nprobe 96: recall@10 0.9960, p50 3286.694 ms, p95 3487.819 ms, p99 3554.765 ms, max 3620.900 ms
  - Route-stage containment from `pipeline-fanout16-route-containment.tsv`:
    - nprobe 8: 1496/2000 truth items contained, 0.7480
    - nprobe 16: 1721/2000 truth items contained, 0.8605
    - nprobe 24: 1834/2000 truth items contained, 0.9170
    - nprobe 32: 1880/2000 truth items contained, 0.9400
    - nprobe 48: 1936/2000 truth items contained, 0.9680
    - nprobe 64: 1969/2000 truth items contained, 0.9845
    - nprobe 96: 1992/2000 truth items contained, 0.9960
  - Pipeline counters:
    - nprobe 8: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 1,236,917, object_bytes_sum 1,008,532,232
    - nprobe 16: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 2,516,097, object_bytes_sum 2,051,510,976
    - nprobe 24: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 3,778,758, object_bytes_sum 3,081,024,144
    - nprobe 32: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 5,066,438, object_bytes_sum 4,130,930,864
    - nprobe 48: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 7,613,053, object_bytes_sum 6,207,309,256
    - nprobe 64: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 10,174,090, object_bytes_sum 8,295,447,760
    - nprobe 96: routing `truncated`, `next_blocker=routing_budget`, candidate_sum 15,340,768, object_bytes_sum 12,508,080,736
  - Interpretation: `recursive_fanout=16` is a small low/mid-nprobe recall improvement with nearly baseline storage and candidate volume. It is not in the same class as boundary replication, and it slightly underperforms baseline at nprobe 96, but it is cheap enough to keep under consideration pending `recursive_fanout=32`.
