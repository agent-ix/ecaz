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

First OFAT lever result, `top_graph_search_list_size=200`, low-nprobe screen:

| nprobe | baseline recall@10 | tgsl200 recall@10 | baseline p50 | tgsl200 p50 | baseline p95 | tgsl200 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.7250 | 244.812 ms | 257.170 ms | 307.294 ms | 327.460 ms |
| 16 | 0.8525 | 0.8525 | 502.956 ms | 518.332 ms | 609.909 ms | 622.913 ms |
| 24 | 0.9045 | 0.9045 | 785.069 ms | 816.473 ms | 915.098 ms | 947.148 ms |
| 32 | 0.9310 | 0.9310 | 1055.659 ms | 1081.118 ms | 1214.091 ms | 1256.546 ms |
| 48 | 0.9645 | 0.9645 | 1633.655 ms | 1680.941 ms | 1836.622 ms | 1903.131 ms |
| 64 | 0.9825 | 0.9825 | 2181.396 ms | 2207.241 ms | 2481.401 ms | 2482.774 ms |
| 96 | 0.9975 | 0.9975 | 3347.935 ms | 3308.038 ms | 3676.094 ms | 3749.160 ms |

`tgsl200` did not improve route containment or recall at any nprobe in the original low-nprobe sweep. Its candidate counts and local object bytes also matched baseline exactly, and storage stayed at a 79.7 MiB SPIRE index. The reviewer correctly flagged that this low-nprobe result is inert by construction for beam analysis because all measured nprobe values are <= the baseline `top_graph_search_list_size=96`.

Corrected high-nprobe `tgsl200` screen:

| nprobe | route-stage containment | recall@10 | p50 latency | p95 latency | candidate rows |
| --- | ---: | ---: | ---: | ---: | ---: |
| 128 | 1.0000 | 1.0000 | 4351.972 ms | 5125.647 ms | 20,000,000 |
| 160 | 1.0000 | 1.0000 | 4364.396 ms | 5296.145 ms | 20,000,000 |
| 200 | 1.0000 | 1.0000 | 4410.790 ms | 4967.462 ms | 20,000,000 |

The corrected `tgsl200` high-nprobe run proves that widening the beam can reach perfect route containment and recall on this 100k local fixture, but it does so by scanning and heap-reranking the full 100k corpus for every query. Routing counters report `effective_nprobe=128`, `beam_width=128`, and no truncation for requested nprobe 128/160/200; all three settings materialize 20.0M candidates across 200 queries. Practical conclusion for Phase 1: `top_graph_search_list_size=200` is useful as a ceiling/diagnostic bound, but not a viable performance tuning lever at this scale unless paired with another change that prevents full-corpus candidate fanout.

Second OFAT lever result, `boundary_replica_count=1`:

| nprobe | baseline recall@10 | bound1 recall@10 | baseline p50 | bound1 p50 | baseline SPIRE candidates | bound1 SPIRE candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.8365 | 244.812 ms | 497.014 ms | 1,232,065 | 2,638,655 |
| 16 | 0.8525 | 0.9235 | 502.956 ms | 950.199 ms | 2,514,557 | 5,243,699 |
| 24 | 0.9045 | 0.9605 | 785.069 ms | 1335.556 ms | 3,816,799 | 7,848,831 |
| 32 | 0.9310 | 0.9735 | 1055.659 ms | 1742.360 ms | 5,165,224 | 10,487,717 |
| 48 | 0.9645 | 0.9870 | 1633.655 ms | 2449.828 ms | 7,795,405 | 15,763,335 |
| 64 | 0.9825 | 0.9940 | 2181.396 ms | 3146.297 ms | 10,420,357 | 21,025,889 |
| 96 | 0.9975 | 0.9995 | 3347.935 ms | 4137.887 ms | 15,506,227 | 31,137,800 |

`boundary_replica_count=1` is a significant recall lever. It improves route containment and final recall at every measured nprobe, with the largest practical gain at low nprobe: +0.1115 recall@10 at nprobe 8 and +0.0710 at nprobe 16. The cost is also clear: SPIRE index size rises from 79.7 MiB to 157.9 MiB, index build time rises from 11.60s to 44.40s, candidate rows roughly double, and coordinator latency rises materially. Route-stage containment still equals final recall for every nprobe, so the measured pipeline remains route-limited; boundary replicas improve routing coverage rather than exposing a downstream loss stage.

Third OFAT lever result, `boundary_replica_count=2`:

| nprobe | baseline recall@10 | bound1 recall@10 | bound2 recall@10 | baseline p50 | bound1 p50 | bound2 p50 | bound2 candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.8365 | 0.8730 | 244.812 ms | 497.014 ms | 696.395 ms | 3,942,723 |
| 16 | 0.8525 | 0.9235 | 0.9465 | 502.956 ms | 950.199 ms | 1227.200 ms | 7,878,446 |
| 24 | 0.9045 | 0.9605 | 0.9745 | 785.069 ms | 1335.556 ms | 1737.343 ms | 11,786,881 |
| 32 | 0.9310 | 0.9735 | 0.9835 | 1055.659 ms | 1742.360 ms | 2181.439 ms | 15,730,984 |
| 48 | 0.9645 | 0.9870 | 0.9925 | 1633.655 ms | 2449.828 ms | 3034.067 ms | 23,657,267 |
| 64 | 0.9825 | 0.9940 | 0.9970 | 2181.396 ms | 3146.297 ms | 3607.115 ms | 31,551,045 |
| 96 | 0.9975 | 0.9995 | 1.0000 | 3347.935 ms | 4137.887 ms | 4553.530 ms | 46,704,408 |

`boundary_replica_count=2` improves route containment again, but with diminishing recall gain versus bound1 and another large cost jump. The largest incremental bound2-over-bound1 gains are +0.0365 recall@10 at nprobe 8 and +0.0230 at nprobe 16; by nprobe 96 the gain is only +0.0005. SPIRE index size rises to 235.9 MiB, index build time rises to 54.47s, and candidate rows at nprobe 96 reach 46.7M across 200 queries. Route-stage containment still exactly equals final recall, so the finding remains clear: boundary replicas buy routing coverage, but the cost curve steepens quickly.

Fourth OFAT lever result, `boundary_replica_count=4`:

| nprobe | baseline recall@10 | bound1 recall@10 | bound2 recall@10 | bound4 recall@10 | bound4 p50 | bound4 p95 | bound4 candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.8365 | 0.8730 | 0.9135 | 1017.592 ms | 1340.120 ms | 6,473,923 |
| 16 | 0.8525 | 0.9235 | 0.9465 | 0.9655 | 1791.907 ms | 2216.811 ms | 13,094,002 |
| 24 | 0.9045 | 0.9605 | 0.9745 | 0.9875 | 2368.554 ms | 2880.358 ms | 19,636,910 |
| 32 | 0.9310 | 0.9735 | 0.9835 | 0.9915 | 2899.310 ms | 3537.148 ms | 26,182,573 |
| 48 | 0.9645 | 0.9870 | 0.9925 | 0.9955 | 3695.809 ms | 4414.522 ms | 39,338,713 |
| 64 | 0.9825 | 0.9940 | 0.9970 | 0.9985 | 4242.798 ms | 5001.764 ms | 52,438,585 |
| 96 | 0.9975 | 0.9995 | 1.0000 | 1.0000 | 4948.452 ms | 5773.607 ms | 77,582,377 |

`boundary_replica_count=4` confirms the knee is before 4 for this local 100k RaBitQ screen. It improves low-nprobe recall versus bound2 (+0.0405 at nprobe 8, +0.0190 at nprobe 16), but bound2 already reaches 1.0000 at nprobe 96 and bound4 adds no high-nprobe recall headroom. The cost is severe: SPIRE index size rises to 392.2 MiB, index bytes/row to 4112.4 B, and nprobe 96 materializes 77.6M candidate rows across 200 queries while heap-reranking 19.3M retained rows. Route-stage containment still exactly equals final recall at every nprobe, so the pipeline remains route-limited; bound4 is evidence for the diminishing-return boundary, not a practical default.

Fifth OFAT lever result, `nlists=316`:

| nprobe | baseline recall@10 | bound2 recall@10 | nlist316 recall@10 | baseline p50 | nlist316 p50 | baseline candidates | nlist316 candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.8730 | 0.7370 | 244.812 ms | 110.945 ms | 1,232,065 | 487,390 |
| 16 | 0.8525 | 0.9465 | 0.8230 | 502.956 ms | 207.869 ms | 2,514,557 | 999,460 |
| 24 | 0.9045 | 0.9745 | 0.8650 | 785.069 ms | 321.769 ms | 3,816,799 | 1,500,025 |
| 32 | 0.9310 | 0.9835 | 0.8975 | 1055.659 ms | 435.530 ms | 5,165,224 | 2,009,214 |
| 48 | 0.9645 | 0.9925 | 0.9220 | 1633.655 ms | 654.295 ms | 7,795,405 | 3,051,404 |
| 64 | 0.9825 | 0.9970 | 0.9445 | 2181.396 ms | 876.194 ms | 10,420,357 | 4,062,261 |
| 96 | 0.9975 | 1.0000 | 0.9720 | 3347.935 ms | 1351.068 ms | 15,506,227 | 6,137,532 |

`nlists=316` is a scan-efficiency/storage lever, not a route-recall fix by itself. It cuts the SPIRE index to 81.8 MiB / 858.1 B per row and cuts candidate fanout by roughly 60% versus baseline at the same nprobe, yielding much lower coordinator latency. But route-stage containment again exactly equals final recall, and recall is worse than baseline from nprobe 16 through 96. Compared with `boundary_replica_count=2`, it is dramatically cheaper but does not rival the recall improvement. Practical Phase 1 conclusion: keep `nlists` in the factorial only if paired with a coverage lever such as boundary replication; do not treat higher `nlists` alone as significant for the route-loss objective.

Sixth OFAT lever result, `nlists=512`:

| nprobe | baseline recall@10 | nlist316 recall@10 | nlist512 recall@10 | nlist512 p50 | nlist512 p95 | nlist512 candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.7370 | 0.6600 | 82.284 ms | 127.963 ms | 329,783 |
| 16 | 0.8525 | 0.8230 | 0.7630 | 145.977 ms | 191.681 ms | 652,296 |
| 24 | 0.9045 | 0.8650 | 0.8125 | 214.549 ms | 269.854 ms | 983,356 |
| 32 | 0.9310 | 0.8975 | 0.8480 | 289.288 ms | 371.181 ms | 1,319,095 |
| 48 | 0.9645 | 0.9220 | 0.8850 | 437.427 ms | 526.012 ms | 1,984,266 |
| 64 | 0.9825 | 0.9445 | 0.9075 | 590.094 ms | 721.693 ms | 2,654,041 |
| 96 | 0.9975 | 0.9720 | 0.9350 | 890.681 ms | 1156.328 ms | 3,991,647 |

`nlists=512` strengthens the negative result for finer leaves as a standalone routing lever. It reduces the SPIRE candidate rows again versus `nlists=316` and keeps index size close to baseline at 84.0 MiB / 881.1 B per row, but route containment and final recall fall at every measured nprobe. At nprobe 96 it is 6.25 recall points below `nlists=316` and 6.25 points below baseline. This suggests the finer-list setting is behaving primarily as a scan-pruning/cost lever under the fixed routing budget, not as a centroid-quality recovery lever. The remaining `nlists=1024` cell should be treated as a boundary check for this degradation curve, not as likely recall recovery.

Seventh OFAT lever result, `nlists=1024`:

| nprobe | baseline recall@10 | nlist316 recall@10 | nlist512 recall@10 | nlist1024 recall@10 | nlist1024 p50 | nlist1024 candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.7370 | 0.6600 | 0.6410 | 83.488 ms | 196,031 |
| 16 | 0.8525 | 0.8230 | 0.7630 | 0.7190 | 118.638 ms | 390,113 |
| 24 | 0.9045 | 0.8650 | 0.8125 | 0.7725 | 161.101 ms | 575,635 |
| 32 | 0.9310 | 0.8975 | 0.8480 | 0.8110 | 191.812 ms | 756,686 |
| 48 | 0.9645 | 0.9220 | 0.8850 | 0.8635 | 272.876 ms | 1,113,024 |
| 64 | 0.9825 | 0.9445 | 0.9075 | 0.8910 | 364.600 ms | 1,474,921 |
| 96 | 0.9975 | 0.9720 | 0.9350 | 0.9215 | 517.387 ms | 2,177,568 |

`nlists=1024` completes the standalone `nlists` boundary check and confirms the degradation curve. Finer leaves monotonically reduce candidate rows and coordinator latency under the fixed nprobe budget, but route-stage containment and final recall continue to fall. At nprobe 96, `nlists=1024` is 7.60 recall points below baseline and 5.05 points below `nlists=316`. This closes the standalone `nlists` screen as a negative recall lever for Task 121: useful as a cost lever to pair with boundary replication or another coverage lever, but not significant by itself for the route-loss objective.

Eighth OFAT lever result, `recursive_fanout`:

`recursive_fanout=0` is not a valid single-factor cell with the Task 121 baseline because `top_graph_enabled=1` requires `recursive_fanout >= 2` during index build. The packet keeps the failed load artifact for provenance, but there is no recall/route-containment result for that invalid combination.

| nprobe | baseline recall@10 | fanout16 recall@10 | baseline p50 | fanout16 p50 | baseline candidates | fanout16 candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0.7250 | 0.7480 | 244.812 ms | 250.995 ms | 1,232,065 | 1,236,917 |
| 16 | 0.8525 | 0.8605 | 502.956 ms | 534.268 ms | 2,514,557 | 2,516,097 |
| 24 | 0.9045 | 0.9170 | 785.069 ms | 797.713 ms | 3,816,799 | 3,778,758 |
| 32 | 0.9310 | 0.9400 | 1055.659 ms | 1073.706 ms | 5,165,224 | 5,066,438 |
| 48 | 0.9645 | 0.9680 | 1633.655 ms | 1626.846 ms | 7,795,405 | 7,613,053 |
| 64 | 0.9825 | 0.9845 | 2181.396 ms | 2187.801 ms | 10,420,357 | 10,174,090 |
| 96 | 0.9975 | 0.9960 | 3347.935 ms | 3286.694 ms | 15,506,227 | 15,340,768 |

`recursive_fanout=16` is a small positive low/mid-nprobe routing lever, not a dominant recall fix. Route-stage containment again equals final recall for every nprobe. The largest gain is +0.0230 recall@10 at nprobe 8, and the effect shrinks to near-neutral by nprobe 48/64 before slightly underperforming baseline at nprobe 96. Candidate rows and index size remain close to baseline, so this is a cheap minor precision tweak worth retaining for the final significant-set decision only if `fanout32` confirms a real trend.

## Artifacts

See `artifacts/manifest.md` for artifact metadata and command provenance.

Truth-cache JSON files are intentionally not committed; they are regenerable caches. The suite manifests/results/logs record the commands and cited result lines.
