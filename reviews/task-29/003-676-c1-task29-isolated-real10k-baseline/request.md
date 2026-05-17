# Task 29 isolated real-10k DiskANN baseline

## Status

Fresh local PG18 measurements are now available for isolated one-index-per-table
surfaces. The earlier shared-table baseline was not sufficient because, after
fixing the `ecaz-cli` KNN query shape, PostgreSQL could choose the HNSW index on
the shared table. This packet uses separate prefixes:

- `task29_diskann_real10k`: one `ec_diskann` index.
- `task29_hnsw_real10k`: one `ec_hnsw` m16 reference index.

Head SHA: `9d4d10ec2e5c54e9e0b79705f92e6fd13e809e82`

## Setup

DiskANN profile and reloptions:

```text
profile=ec_diskann
graph_degree=32
build_list_size=100
alpha=1.2
```

Corpus:

```text
target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv
target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv
rows=10000
queries=200
dim=1536
bits=4
seed=42
```

The source TSV sibling manifest still names the original prepared prefix
`ec_hnsw_real_10k`; the isolated loads therefore used the checked-in loader's
explicit `--allow-manifest-mismatch` flag. The data hashes are unchanged.

## Correctness Of Measurement Path

`artifacts/explain-diskann-isolated-q1.log` confirms the DiskANN table uses the
DiskANN index:

```text
Index Scan using task29_diskann_real10k_idx on task29_diskann_real10k_corpus
Buffers: shared hit=186 read=736
Execution Time: 68.055 ms
```

This is a local warm-up run before the recall and latency sweeps. No cache flush
was performed; latency results should be read as local warm-cache-after-recall
measurements.

## Load And Build Timing

DiskANN (`artifacts/load-diskann-isolated.log`):

```text
copied corpus table task29_diskann_real10k_corpus in 9.70s
encoded corpus table task29_diskann_real10k_corpus in 4.21s
copied queries table task29_diskann_real10k_queries in 213.61ms
built task29_diskann_real10k_idx in 491.05s
completed prefix task29_diskann_real10k in 520.66s
```

HNSW m16 reference (`artifacts/load-hnsw-isolated.log`):

```text
copied corpus table task29_hnsw_real10k_corpus in 9.67s
encoded corpus table task29_hnsw_real10k_corpus in 4.69s
copied queries table task29_hnsw_real10k_queries in 265.12ms
built task29_hnsw_real10k_m16_idx in 89.14s
completed prefix task29_hnsw_real10k in 119.72s
```

## Recall Sweep

DiskANN (`artifacts/recall-diskann-isolated-table.log`):

| list_size | recall@10 | NDCG@10 | mean query |
| ---: | ---: | ---: | ---: |
| 64 | 0.9280 | 0.9959 | 70.96 ms |
| 128 | 0.9310 | 0.9966 | 74.01 ms |
| 200 | 0.9315 | 0.9966 | 84.70 ms |
| 400 | 0.9315 | 0.9966 | 126.73 ms |
| 800 | 0.9315 | 0.9966 | 268.90 ms |

HNSW m16 reference (`artifacts/recall-hnsw-isolated-table.log`):

| ef_search | recall@10 | NDCG@10 | mean query |
| ---: | ---: | ---: | ---: |
| 64 | 0.9305 | 0.9814 | 18.63 ms |
| 128 | 0.9645 | 0.9967 | 28.04 ms |
| 200 | 0.9700 | 0.9993 | 37.42 ms |
| 400 | 0.9710 | 0.9994 | 63.30 ms |
| 800 | 0.9720 | 0.9995 | 119.12 ms |

## Latency And Memory

DiskANN, 200 iterations per point, concurrency 1
(`artifacts/latency-diskann-isolated-table.log`):

| list_size | mean | p50 | p95 | p99 | HWM |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 61.9 ms | 61.7 ms | 65.4 ms | 68.0 ms | 82632 KiB |
| 128 | 72.2 ms | 72.5 ms | 77.9 ms | 81.9 ms | 83752 KiB |
| 200 | 85.2 ms | 84.1 ms | 95.6 ms | 104.9 ms | 83432 KiB |
| 400 | 126.3 ms | 125.7 ms | 142.4 ms | 148.8 ms | 84072 KiB |
| 800 | 269.4 ms | 267.0 ms | 301.3 ms | 316.2 ms | 84232 KiB |

HNSW m16, 200 iterations per point, concurrency 1
(`artifacts/latency-hnsw-isolated-table.log`):

| ef_search | mean | p50 | p95 | p99 | HWM |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 18.9 ms | 17.9 ms | 25.1 ms | 29.6 ms | 77352 KiB |
| 128 | 28.1 ms | 26.9 ms | 36.1 ms | 44.4 ms | 78152 KiB |
| 200 | 36.8 ms | 35.7 ms | 43.4 ms | 51.2 ms | 78792 KiB |
| 400 | 63.2 ms | 62.3 ms | 70.2 ms | 76.0 ms | 79112 KiB |
| 800 | 121.1 ms | 118.9 ms | 144.2 ms | 158.6 ms | 80128 KiB |

## Storage

DiskANN (`artifacts/storage-diskann-isolated.log`):

```text
task29_diskann_real10k_idx  ec_diskann  {graph_degree=32,build_list_size=100,alpha=1.2}  4.7 MiB  494.0 B/row
total relation size: 164.5 MiB
```

HNSW m16 (`artifacts/storage-hnsw-isolated.log`):

```text
task29_hnsw_real10k_m16_idx  ec_hnsw  {m=16,ef_construction=128,build_source_column=source}  13.0 MiB  1366.4 B/row
total relation size: 172.9 MiB
```

## Recommendation

Do not tune scan `list_size` first. The DiskANN recall curve is effectively
flat after `list_size=128`, while latency increases sharply. The first useful
optimization is build-side graph quality: sweep or inspect `graph_degree`,
`build_list_size`, `alpha`, and Vamana pruning/candidate selection.

Landing blocker: at this setting DiskANN is much smaller than HNSW, but it is
not competitive on local real-10k quality or latency. It builds about 5.5x
slower than the HNSW m16 reference, has lower recall ceiling, and is slower at
comparable recall points. I would only land this branch as experimental unless
the next build-side tuning pass materially improves recall and build time.
