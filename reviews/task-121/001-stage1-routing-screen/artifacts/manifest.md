# Task 121 Stage 1 Routing Screen Artifact Manifest

- Head SHA: `f63bb98894f3e677c31a7042b26d72939dc42ed4`
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
