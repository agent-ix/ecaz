# Task 121 Stage 1 Routing Screen - Local Baseline Evidence

## Scope

This packet carries the Task 121 Stage 1 local 100k RaBitQ baseline setup plus completed local baseline pipeline evidence. It does not claim the full OFAT matrix is complete.

The suite config now:

- scopes storage-format variants to RaBitQ baseline plus TurboQuant only;
- adds explicit `truth_cache_file` wiring to all `spire-pipeline` steps so exact truth is generated once and reused;
- adds bounded q20/nprobe96 baseline steps for fast local sanity evidence before widening to q200/full sweeps;
- emits pipeline funnel and stage-containment JSONL incrementally during long pipeline runs.

## Completed Local Evidence

Host/database:

- PG18 local socket: `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench_task121`
- Corpus: `data/staged-current/ec_real_100k_corpus.tsv`
- Queries: `data/staged-current/ec_real_100k_queries.tsv`
- Prefix/index: `t121_s1_100k_baseline`, `t121_s1_100k_baseline_idx`
- Storage format: RaBitQ

Baseline load/storage completed:

- Load copied corpus in 95.55s, encoded corpus in 36.24s, copied queries in 980.82ms, built index in 11.60s.
- Storage total: 1.6 GiB; SPIRE index: 79.7 MiB; index bytes/row: 835.8 B.

Bounded q20/nprobe96 result:

- `truth-cache-100k-q20-k10`: recall@10 1.0000, CI95 low 0.9812, mean q-time 3363.50 ms.
- `pipeline-baseline-q20-n96`: recall@10 1.0000, p50 3238.948 ms, p95 3411.542 ms, p99/max 3909.826 ms.
- Pipeline counters at nprobe96/q20: route_sum 1920, candidate_sum 1,522,002, heap_rerank_sum 1,522,002.
- Stage containment for the sampled q20 run showed final top-k containment 10/10 for the sampled rows; routing stage status was `truncated` with `next_blocker=routing_budget`, so route-budget pressure is real even when final recall is perfect at nprobe96.

Full q200/seven-sweep baseline result:

| nprobe | recall@10 | route-stage containment | p50 latency | p95 latency | candidate rows |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.7250 | 244.812 ms | 307.294 ms | 1,232,065 |
| 16 | 0.8525 | 0.8525 | 502.956 ms | 609.909 ms | 2,514,557 |
| 24 | 0.9045 | 0.9045 | 785.069 ms | 915.098 ms | 3,816,799 |
| 32 | 0.9310 | 0.9310 | 1055.659 ms | 1214.091 ms | 5,165,224 |
| 48 | 0.9645 | 0.9645 | 1633.655 ms | 1836.622 ms | 7,795,405 |
| 64 | 0.9825 | 0.9825 | 2181.396 ms | 2481.401 ms | 10,420,357 |
| 96 | 0.9975 | 0.9975 | 3347.935 ms | 3676.094 ms | 15,506,227 |

Key baseline finding: route-stage containment equals final recall for every measured nprobe. The baseline loses truth at route selection, not in later placement, prefetch, candidate materialization, or heap rerank stages. Every routing row reported `status=truncated` and `next_blocker=routing_budget`, so Task 121 Phase 1 should prioritize levers that directly change routing coverage before tuning downstream scan costs.

The original q200/seven-sweep attempt was canceled after more than 33 minutes because artifacts were opaque during execution. After the incremental JSONL runner change captured in `reviews/task-121/003-spire-pipeline-incremental-jsonl/`, the same baseline completed locally and wrote packet-local evidence throughout the run.

First OFAT lever result, `top_graph_search_list_size=200`:

| nprobe | baseline recall@10 | tgsl200 recall@10 | baseline p50 | tgsl200 p50 | baseline p95 | tgsl200 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.7250 | 244.812 ms | 257.170 ms | 307.294 ms | 327.460 ms |
| 16 | 0.8525 | 0.8525 | 502.956 ms | 518.332 ms | 609.909 ms | 622.913 ms |
| 24 | 0.9045 | 0.9045 | 785.069 ms | 816.473 ms | 915.098 ms | 947.148 ms |
| 32 | 0.9310 | 0.9310 | 1055.659 ms | 1081.118 ms | 1214.091 ms | 1256.546 ms |
| 48 | 0.9645 | 0.9645 | 1633.655 ms | 1680.941 ms | 1836.622 ms | 1903.131 ms |
| 64 | 0.9825 | 0.9825 | 2181.396 ms | 2207.241 ms | 2481.401 ms | 2482.774 ms |
| 96 | 0.9975 | 0.9975 | 3347.935 ms | 3308.038 ms | 3676.094 ms | 3749.160 ms |

`tgsl200` did not improve route containment or recall at any nprobe. Its candidate counts and local object bytes also matched baseline exactly, and storage stayed at a 79.7 MiB SPIRE index. Practical conclusion for Phase 1: increasing `top_graph_search_list_size` from 96 to 200 is not a significant lever for this 100k local screen.

## Artifacts

See `artifacts/manifest.md` for artifact metadata and command provenance.

Truth-cache JSON files are intentionally not committed; they are regenerable caches. The suite manifests/results/logs record the commands and cited result lines.
