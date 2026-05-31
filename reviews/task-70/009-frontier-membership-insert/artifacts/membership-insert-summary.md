# Task 70 Packet 009 Membership Insert Summary

- Code head measured: `31de2206a6eda1578d48d90e70661eaf24108fda`
- Suite: `task70-diskann-frontier-membership-insert-real10k`
- Fixture: real10K DBPedia staged corpus, PG18, pq_fastscan, graph_degree=32, build_list_size=100, alpha=1.2, rerank_budget=64, top_k=10
- Storage surface: isolated packet-local prefix `task70_009_diskann`
- Baseline packet: `reviews/task-70/008-frontier-subtiming-profile/`

## Code Slice

The scan loop previously tested a new neighbor with `HashSet::contains` and then inserted the same TID inside `push_frontier_entry`, so every first-seen neighbor paid two hash-table membership operations. This slice changes the neighbor loop to use `HashSet::insert` as the membership test and makes `push_frontier_entry` only push the already-marked candidate into the heap.

This does not change traversal order, candidate scoring, tombstone handling, rerank order, or emitted results.

## Suite Results

| lane | packet 008 recall | packet 009 recall | packet 008 recall q-time | packet 009 recall q-time |
| --- | ---: | ---: | ---: | ---: |
| L64 | 0.9965 | 0.9965 | 0.68 ms | 0.66 ms |
| L200 | 0.9975 | 0.9975 | 0.96 ms | 0.90 ms |

Recall remains unchanged and at the Task 70 floor.

## pgvectorscale Compare Step

This is the cleanest latency comparison in the suite because it does not enable scan profile NOTICE output.

| lane | packet 008 ec mean | packet 009 ec mean | delta | packet 008 ec p95 | packet 009 ec p95 | delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| L64 | 0.69 ms | 0.66 ms | -4.3% | 0.82 ms | 0.80 ms | -2.4% |
| L200 | 0.97 ms | 0.91 ms | -6.2% | 1.14 ms | 1.08 ms | -5.3% |

Full compare:

| engine | recall@k | mean | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| ec_diskann L64 | 0.9965 | 0.66 ms | 0.80 ms | 0.94 ms |
| pgvectorscale L64 | 0.9955 | 0.66 ms | 0.87 ms | 1.04 ms |
| ec_diskann L200 | 0.9975 | 0.91 ms | 1.08 ms | 1.15 ms |
| pgvectorscale L200 | 1.0000 | 1.15 ms | 1.41 ms | 1.55 ms |

## Profiled Latency Step

This step enables one scan-profile NOTICE per scan, so it is diagnostic rather than an optimized latency baseline.

| lane | packet 008 mean | packet 009 mean | delta | packet 008 p95 | packet 009 p95 | delta | packet 008 p99 | packet 009 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| L64 | 0.83 ms | 0.76 ms | -8.4% | 1.13 ms | 0.87 ms | -23.0% | 1.60 ms | 0.99 ms |
| L200 | 1.29 ms | 1.25 ms | -3.1% | 1.51 ms | 1.49 ms | -1.3% | 1.59 ms | 1.63 ms |

## Profile NOTICE Means

Each raw profile step emitted 200 NOTICE rows. Times are microseconds.

| field | packet 008 L64 mean | packet 009 L64 mean | packet 008 L200 mean | packet 009 L200 mean |
| --- | ---: | ---: | ---: | ---: |
| `frontier_us` | 401.60 | 370.09 | 920.04 | 842.88 |
| `frontier_candidate_heap_us` | 4.13 | 0.17 | 10.96 | 0.32 |
| `frontier_visited_set_us` | 0.23 | 4.08 | 1.06 | 10.78 |
| `frontier_neighbor_iter_us` | 0.09 | 0.25 | 0.79 | 0.30 |
| `frontier_retained_insert_us` | 0.01 | 0.00 | 0.00 | 0.14 |
| `exact_rerank_us` | 90.11 | 91.80 | 93.24 | 96.17 |
| `total_us` | 513.30 | 481.90 | 1037.58 | 960.47 |

The sub-bucket shift is expected: membership insertion moved out of the helper whose timing was charged to `frontier_candidate_heap_us` and is now inside the loop timing charged to `frontier_visited_set_us`. The stable comparison is the enclosing `frontier_us` and `total_us`: L64 frontier mean improved 401.60 -> 370.09 us (-7.8%), and L200 improved 920.04 -> 842.88 us (-8.4%).

## Operation Counts

Operation counts remain unchanged from packet 008, as expected. The slice removes one hash-table lookup for first-seen neighbors; it does not change how many neighbors are considered or how many candidates enter the heap.

| field | L64 mean | L64 p95 | L200 mean | L200 p95 |
| --- | ---: | ---: | ---: | ---: |
| `frontier_candidate_heap_ops` | 893.67 | 1117 | 1990.26 | 2530 |
| `frontier_visited_set_ops` | 1751.26 | 1973 | 5256.80 | 5704 |
| `frontier_neighbor_slots` | 1751.26 | 1973 | 5256.80 | 5704 |
| `frontier_retained_inserts` | 67.15 | 72 | 201.95 | 205 |

## Interpretation

This is a modest but real win: recall is unchanged, compare latency improves at both L64 and L200, and profiled frontier/total means improve even with NOTICE overhead. The effect size is small enough that a reviewer should treat this as a useful local cleanup, not as a final Task 70 closeout.
