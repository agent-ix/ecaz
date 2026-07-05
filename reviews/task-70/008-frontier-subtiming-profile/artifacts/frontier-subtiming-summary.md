# Task 70 Packet 008 Frontier Sub-Timing Summary

- Code head measured: `f1665fe5760f8eec569bb2560841ed91f270ed09`
- Suite: `task70-diskann-frontier-subtiming-real10k`
- Fixture: real10K DBPedia staged corpus, PG18, pq_fastscan, graph_degree=32, build_list_size=100, alpha=1.2, rerank_budget=64, top_k=10
- Storage surface: isolated packet-local prefix `task70_008_diskann`
- Run timestamp: `2026-05-31T20:38:49Z` from `suite-manifest.json`

## Validation and Compare

| lane | recall@k | recall mean q-time | profiled latency mean | profiled latency p95 | profiled latency p99 | pgvectorscale compare mean | pgvectorscale p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| L64 | 0.9965 | 0.68 ms | 0.83 ms | 1.13 ms | 1.60 ms | 0.65 ms | 0.84 ms |
| L200 | 0.9975 | 0.96 ms | 1.29 ms | 1.51 ms | 1.59 ms | 1.21 ms | 1.59 ms |

The pgvectorscale comparison step, which does not enable `ec_diskann.scan_profile_notice`, measured `ec_diskann` at 0.69 ms mean / 0.82 ms p95 for L64 and 0.97 ms mean / 1.14 ms p95 for L200. The profiled latency step is slower because it emits one NOTICE per scan and should be treated as instrumentation evidence, not as an optimized latency baseline.

Recall remains at the Task 70 floor: L64 0.9965 and L200 0.9975.

## Profile Notice Means and Tails

Each profile-notice run emits 200 rows. Times are microseconds.

| field | L64 mean | L64 p95 | L64 p99 | L200 mean | L200 p95 | L200 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| setup_us | 13.22 | 16 | 18 | 13.88 | 17 | 22 |
| graph_read_decode_us | 3.35 | 9 | 27 | 5.32 | 21 | 42 |
| frontier_us | 401.60 | 484 | 530 | 920.04 | 1113 | 1182 |
| frontier_candidate_heap_us | 4.13 | 9 | 10 | 10.96 | 20 | 30 |
| frontier_visited_set_us | 0.23 | 0 | 8 | 1.06 | 8 | 12 |
| frontier_neighbor_iter_us | 0.09 | 0 | 0 | 0.79 | 6 | 11 |
| frontier_retained_insert_us | 0.01 | 0 | 0 | 0.00 | 0 | 0 |
| heap_prefetch_us | 4.17 | 5 | 7 | 4.20 | 5 | 6 |
| exact_rerank_us | 90.11 | 146 | 194 | 93.24 | 137 | 159 |
| total_us | 513.30 | 628 | 724 | 1037.58 | 1257 | 1384 |

The new sub-timers confirm that explicit candidate-heap time is small relative to the full frontier residual in this build: 4.13 us mean at L64 and 10.96 us mean at L200. The visited-set, neighbor-iteration, and retained-insert timers usually quantize to zero at microsecond granularity, so they do not explain the remaining frontier time by wall clock alone.

## Frontier Operation Counts

| field | L64 mean | L64 p95 | L64 p99 | L200 mean | L200 p95 | L200 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| graph_read_count | 758.37 | 984 | 1057 | 1585.36 | 2127 | 2253 |
| prefilter_count | 758.37 | 984 | 1057 | 1585.36 | 2127 | 2253 |
| frontier_candidate_heap_ops | 893.67 | 1117 | 1188 | 1990.26 | 2530 | 2668 |
| frontier_visited_set_ops | 1751.26 | 1973 | 2146 | 5256.80 | 5704 | 5754 |
| frontier_neighbor_slots | 1751.26 | 1973 | 2146 | 5256.80 | 5704 | 5754 |
| frontier_retained_inserts | 67.15 | 72 | 77 | 201.95 | 205 | 209 |
| rerank_count | 64.00 | 64 | 64 | 64.00 | 64 | 64 |
| result_count | 64.00 | 64 | 64 | 64.00 | 64 | 64 |

The counts are the useful slice signal: L200 does about 2.23x candidate-heap operations, 3.00x visited-set checks, and 3.00x neighbor-slot iterations versus L64. Retained inserts scale with `list_size` as expected, but explicit retained insertion time is below useful microsecond resolution.

## Interpretation

Packet 004's retained-neighbor vector continues to hold recall and compare latency. Packet 008 adds the requested frontier subfields, but the first-pass wall-time split is too fine-grained for the fastest inner operations. The next frontier performance slice should not target the candidate heap first based on this data. The larger scaling signal is the volume of visited-set and neighbor-slot work, so the next P0 candidate is to reduce duplicated neighbor/visited work or reframe the timing into coarser loop buckets before choosing another data-structure rewrite.
