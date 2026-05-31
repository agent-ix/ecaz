# Task 70 Final Measurement Summary

- Head measured: `5aad1539eb285153a44d59147e7f12fdde737ebd`
- Code head in the measured tree: `1c0de8436e1a67421a7a00d94006123a06f2a302`
- Docs commit: `5f7f83b35b4b10755fd430e887013cb821e27fbb`
- Suite: `task70-diskann-final-real10k`
- Fixture: real10K DBPedia staged corpus, PG18, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2`, `rerank_budget=64`, `top_k=10`
- Storage surface: isolated packet-local prefix `task70_012_diskann`
- Baseline packets: Phase 1 packet `reviews/task-70/003-phase1-suite-config/`, accepted frontier slice packet `reviews/task-70/009-frontier-membership-insert/`, and historical docs baseline `reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/`

## Recall Floors

Task 70 required L64 within 0.5 pp of `0.9965` and L200 within 0.5 pp of `0.9970`.

| list_size | required floor | final recall@10 | result |
| ---: | ---: | ---: | --- |
| 64 | 0.9915 | 0.9965 | pass |
| 200 | 0.9920 | 0.9975 | pass |

## Clean Cross-Engine Compare

This is the primary latency comparison because it does not enable scan-profile NOTICE output.

| engine | L/search_list | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | 64 | 0.9965 | 0.64 ms | 0.63 ms | 0.77 ms | 0.91 ms |
| `pgvectorscale` | 64 | 0.9960 | 0.60 ms | 0.60 ms | 0.72 ms | 0.89 ms |
| `ec_diskann` | 200 | 0.9975 | 0.88 ms | 0.87 ms | 1.02 ms | 1.13 ms |
| `pgvectorscale` | 200 | 1.0000 | 1.14 ms | 1.13 ms | 1.40 ms | 1.47 ms |

Residual gap / closure:

- L64: `ec_diskann` is `0.04 ms` mean slower than `pgvectorscale` (`6.7%`) and `0.02 ms` p99 slower (`2.2%`) while recall is `0.0005` higher.
- L200: `ec_diskann` is `0.26 ms` mean faster than `pgvectorscale` (`22.8%`) and `0.34 ms` p99 faster (`23.1%`) while recall is `0.0025` lower but above the Task 70 floor.
- Versus the historical Task 32 docs row at L64, `ec_diskann` moved from `2.14 ms` mean / `2.67 ms` p99 to `0.64 ms` mean / `0.91 ms` p99 (`-70.1%` mean, `-65.9%` p99).

Build and size:

- `ec_diskann` build: `6.86 s`, index size `4,939,776 B`.
- `pgvectorscale` compare build: `1.54 s`, index size `5,136,384 B`.

## Repeated Phase Split

Each raw profile step emitted 200 `ec_diskann_scan_profile` NOTICE rows. Times are microseconds.

| list_size | field | mean | p50 | p95 | p99 |
| ---: | --- | ---: | ---: | ---: | ---: |
| 64 | `frontier_us` | 366.23 | 364 | 443 | 501 |
| 64 | `exact_rerank_us` | 87.07 | 78 | 138 | 159 |
| 64 | `total_us` | 475.19 | 466 | 571 | 696 |
| 200 | `frontier_us` | 844.38 | 846 | 1022 | 1089 |
| 200 | `exact_rerank_us` | 91.77 | 87 | 137 | 162 |
| 200 | `total_us` | 957.56 | 947 | 1158 | 1295 |

Operation counts:

| list_size | field | mean | p95 |
| ---: | --- | ---: | ---: |
| 64 | `graph_read_count` | 758.37 | 984 |
| 64 | `frontier_candidate_heap_ops` | 893.67 | 1117 |
| 64 | `frontier_visited_set_ops` | 1751.26 | 1973 |
| 64 | `frontier_retained_inserts` | 67.15 | 72 |
| 200 | `graph_read_count` | 1585.36 | 2127 |
| 200 | `frontier_candidate_heap_ops` | 1990.26 | 2530 |
| 200 | `frontier_visited_set_ops` | 5256.80 | 5704 |
| 200 | `frontier_retained_inserts` | 201.95 | 205 |

Compared with the Phase 1 ranking, frontier maintenance remains the dominant profiled phase at both L values. The accepted packet 009 membership slice is the only post-ranking P0 frontier change that cleared the measurement gate; the exact heap rerank retention attempt was measured negative and shelved in packet 006.

## P0 Slice Disposition

| P0 area | disposition | evidence |
| --- | --- | --- |
| Frontier / candidate management | Landed packet 004 neighbor retention and packet 009 duplicate membership lookup removal; packet 005 retained heap reverted and shelved. | Packets `004`, `005`, and `009`; packet 009 cleared the 5% gate at L200 and preserved recall. |
| Exact heap rerank fetch/detoast | Packet 006 result retention measured negative and was reverted/shelved. | Packet `006`, accepted reviewer feedback. |
| Graph read/decode cache, binary prefilter tuning, result materialization | Shelved by Phase 1 ranking because each was below P0 threshold on this warmed local fixture. | Packet `003`, accepted reviewer feedback. |

## Validation State

- Packet 011 recorded clean `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`.
- Packet 011 also recorded clean focused DiskANN scan tests and `cargo fmt --check`.
- No Task 70 code slice added new `unsafe` blocks; the accepted scan-kernel changes are safe Rust changes to frontier bookkeeping and profile plumbing.
- `docs/benchmarks.md` now includes the Task 70 cross-engine closeout row and updated residual gap narrative.
