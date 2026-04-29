# Task 29 DiskANN alpha=2.0 probe

## Status

This packet probes one build-side tuning lever after the isolated real-10k
baseline in `review/676-c1-task29-isolated-real10k-baseline`: raise DiskANN
`alpha` from `1.2` to `2.0` while keeping:

```text
profile=ec_diskann
graph_degree=32
build_list_size=100
```

Head SHA: `ce9ccf9ac5838f231d12a978fc6d94994c874f40`

## Setup

Prefix:

```text
task29_diskann_a20_real10k
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
`ec_hnsw_real_10k`; this isolated load therefore used the checked-in loader's
explicit `--allow-manifest-mismatch` flag.

## Correctness Of Measurement Path

`artifacts/explain-diskann-alpha20-q1.log` confirms the alpha=2.0 table uses the
DiskANN index at `list_size=64`:

```text
Index Scan using task29_diskann_a20_real10k_idx on task29_diskann_a20_real10k_corpus
Buffers: shared hit=922
Execution Time: 74.652 ms
```

No cache flush was performed; these are local warm-cache-after-build/recall
measurements.

## Load And Build Timing

DiskANN alpha=2.0 (`artifacts/load-diskann-alpha20.log`):

```text
copied corpus table task29_diskann_a20_real10k_corpus in 9.71s
encoded corpus table task29_diskann_a20_real10k_corpus in 4.09s
copied queries table task29_diskann_a20_real10k_queries in 211.57ms
built task29_diskann_a20_real10k_idx in 1039.46s
completed prefix task29_diskann_a20_real10k in 1069.02s
```

Baseline alpha=1.2 from packet 676 built in `491.05s` and completed in
`520.66s`; alpha=2.0 therefore made the build about 2.1x slower on this corpus.

## Recall Sweep

DiskANN alpha=2.0 (`artifacts/recall-diskann-alpha20-table.log`):

| list_size | recall@10 | NDCG@10 | mean query |
| ---: | ---: | ---: | ---: |
| 64 | 0.6700 | 0.8494 | 67.07 ms |
| 128 | 0.8265 | 0.9415 | 79.06 ms |
| 200 | 0.8610 | 0.9620 | 91.97 ms |
| 400 | 0.8880 | 0.9751 | 133.44 ms |
| 800 | 0.9265 | 0.9952 | 276.83 ms |

Baseline alpha=1.2 from packet 676:

| list_size | recall@10 | NDCG@10 | mean query |
| ---: | ---: | ---: | ---: |
| 64 | 0.9280 | 0.9959 | 70.96 ms |
| 128 | 0.9310 | 0.9966 | 74.01 ms |
| 200 | 0.9315 | 0.9966 | 84.70 ms |
| 400 | 0.9315 | 0.9966 | 126.73 ms |
| 800 | 0.9315 | 0.9966 | 268.90 ms |

## Latency And Memory

DiskANN alpha=2.0, 200 iterations per point, concurrency 1
(`artifacts/latency-diskann-alpha20-table.log`):

| list_size | mean | p50 | p95 | p99 | HWM |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 67.1 ms | 66.2 ms | 75.5 ms | 77.8 ms | 112072 KiB |
| 128 | 80.8 ms | 79.1 ms | 102.9 ms | 128.1 ms | 112552 KiB |
| 200 | 91.1 ms | 90.5 ms | 103.4 ms | 110.5 ms | 112712 KiB |
| 400 | 133.6 ms | 131.7 ms | 156.2 ms | 168.3 ms | 113032 KiB |
| 800 | 277.3 ms | 272.6 ms | 325.2 ms | 337.7 ms | 112232 KiB |

## Storage

DiskANN alpha=2.0 (`artifacts/storage-diskann-alpha20.log`):

```text
task29_diskann_a20_real10k_idx  ec_diskann  {graph_degree=32,build_list_size=100,alpha=2.0}  4.7 MiB  494.0 B/row
total relation size: 164.5 MiB
```

This is effectively unchanged from the alpha=1.2 baseline index size.

## Recommendation

Do not pursue higher `alpha` as the first Task 29 optimization. At `alpha=2.0`,
build time more than doubled, recall was worse at every measured `list_size`,
latency did not improve, and index size stayed the same.

The next optimization should inspect Vamana candidate generation and pruning
behavior rather than widening `alpha`. In particular, verify whether the current
two-pass insertion/prune loop is discarding useful candidates or building a
graph with poor navigability before spending more time on wider scan-side
`list_size` or larger build-side `alpha`.
