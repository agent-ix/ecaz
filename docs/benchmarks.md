# Benchmarks

These results are local development measurements on the DBpedia OpenAI
embeddings corpus where noted. They are useful for engineering decisions and
review packets, but they are not product benchmark claims. Product claims need
dedicated benchmark hardware with controlled cache state, memory, storage, and
repeatability.

## HNSW Baseline

Measured on the [DBpedia OpenAI embeddings corpus](recall-methodology.md)
(1536-dimensional `text-embedding-3-large` embeddings).

| Corpus | Configuration | Recall@10 |
| --- | --- | ---: |
| 10K | `ec_hnsw`, `m = 8`, sweep | 97.1% - 97.5% |
| 50K | `ec_hnsw`, `m = 8`, sweep | 92.6% - 95.2% |

NFR-003 recall targets:

| Configuration | Target |
| --- | ---: |
| `m = 8`, `ef_search = 128` | >= 89% |
| `m = 8`, `ef_search = 200` | >= 93% |
| `m = 16`, `ef_search = 200` | >= 97% |

NFR-001 latency targets for top-10 query on 50K vectors:

| Percentile | Target |
| --- | ---: |
| p50 | < 5 ms |
| p99 | < 15 ms |

## IVF Local Results

Task 28 landed the local IVF v1 access method and competitive-substrate lane.
The current recommendation keeps `storage_format = 'auto'` unchanged, while
using explicit `storage_format = 'pq_fastscan', pq_group_size = 8` for larger
high-dimensional IVF surfaces where speed and index size dominate.

10K and 25K matched shape:
`nlists = 64`, `nprobe = 48`, `rerank = 'heap_f32'`,
`rerank_width = 750`.

| Corpus | IVF profile | Recall@10 | Recall@100 | p50 | p95 | p99 | HWM | Index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10K | TurboQuant | 1.0000 | 0.9966 | 130.6 ms | 231.6 ms | 267.9 ms | 109600 kB | 9,641,984 B |
| 10K | PQ-FastScan g8 | 0.9910 | 0.9360 | 77.3 ms | 80.4 ms | 82.2 ms | 137244 kB | 2,506,752 B |
| 10K | RaBitQ | 1.0000 | 0.9930 | 344.2 ms | 401.3 ms | 413.1 ms | 68212 kB | 9,641,984 B |
| 25K | TurboQuant | 0.9990 | 0.9929 | 284.5 ms | 402.4 ms | 441.5 ms | 155540 kB | 23,289,856 B |
| 25K | PQ-FastScan g8 | 0.9940 | 0.9256 | 116.8 ms | 123.7 ms | 125.7 ms | 156112 kB | 5,300,224 B |
| 25K | RaBitQ | 1.0000 | 0.9915 | 775.7 ms | 835.6 ms | 858.8 ms | 92996 kB | 23,519,232 B |

100K selected point:
`storage_format = 'pq_fastscan'`, `pq_group_size = 8`, `nlists = 128`,
`nprobe = 48`, `rerank = 'heap_f32'`, `rerank_width = 500`.

| Corpus | Recall@10 | Recall@100 | p50 | p95 | p99 | HWM | Index size | Build time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100K | 0.9920 | 0.9552 | 173.4 ms | 225.4 ms | 242.9 ms | 157108 kB | 19,791,872 B | 216.789 s |

990K local directional point:

| Corpus | Recall@10 | Recall@100 | p50 | p95 | p99 | HWM | Index size | Build time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 990K | 0.9860 | 0.9509 | 1029.2 ms | 1169.1 ms | 1224.4 ms | 162636 kB | 177 MB | 33:53.835 |

The 990K row is directional local evidence, not a product claim. Fresh exact
990K fills and long HNSW reference rebuilds were explicitly deferred from the
local desktop gate.

Source packets:

- `review/30145-task28-ivf-a10-current-closure/`
- `review/30119-task28-ivf-a9-100k-current-build/`
- `review/30131-task28-ivf-current-gate-status/`
- `review/30151-task28-ivf-local-landing-status/`

## DiskANN Local Results

Task 29 landed the initial DiskANN/Vamana tuning lane. Final local PG18
release-mode readiness used an isolated real-10K surface.

Build and storage:

| Engine | Configuration | Build time | Index size |
| --- | --- | ---: | ---: |
| `ec_diskann` | `graph_degree = 32`, `build_list_size = 100`, `alpha = 1.2` | 14.59 s | 4,939,776 B |
| `pgvectorscale` | `diskann`, `num_neighbors = 32`, `search_list_size = 100`, `max_alpha = 1.2` | 5.72 s | 5,136,384 B |
| `ec_hnsw` | `m = 32`, `ef_construction = 100`, source build | 5.77 s | 15,130,624 B |

Recall/latency sweep:

| Tuning | `ec_diskann` recall / mean / p99 | `pgvectorscale` recall / mean / p99 | `ec_hnsw` recall / mean / p99 |
| ---: | ---: | ---: | ---: |
| 64 | 0.9965 / 7.80 ms / 10.3 ms | 0.9955 / 3.48 ms / 4.49 ms | 0.9695 / 2.91 ms / 4.78 ms |
| 128 | 0.9965 / 7.79 ms / 10.2 ms | 0.9990 / 5.81 ms / 6.74 ms | 0.9710 / 4.75 ms / 6.83 ms |
| 200 | 0.9970 / 7.98 ms / 10.3 ms | 1.0000 / 8.50 ms / 10.2 ms | 0.9710 / 6.75 ms / 8.58 ms |
| 400 | 0.9970 / 8.49 ms / 10.8 ms | 1.0000 / 17.3 ms / 22.2 ms | 0.9715 / 13.0 ms / 18.0 ms |
| 800 | 0.9975 / 9.34 ms / 12.9 ms | 1.0000 / 30.1 ms / 33.7 ms | 0.9715 / 25.5 ms / 41.1 ms |

`ec_diskann` met the Task 29d local build stop condition and stayed near exact
recall. `pgvectorscale` remains the low-tuning latency reference; `ec_diskann`
beats it from tuning value 200 upward on this local surface.

Source packet: `review/11109-task29d-final-readiness/`.

## Storage

| Metric | Value |
| --- | ---: |
| Raw fp32, 1536 dimensions | 6,144 bytes |
| `tqvector` 4-bit artifact | 783 bytes |
| Compression ratio | 7.85x |
| Tuples per 8KB page | about 9 vs about 1 for fp32 |

## Running Benchmarks

### Criterion microbenchmarks

```bash
make bench
make bench-quant_score
```

### Instruction-count benchmarks

Requires valgrind:

```bash
make bench-iai
```

### SQL benchmarks

Requires PostgreSQL with the extension installed:

```bash
make bench-sql-latency
make bench-storage
make bench-recall-sql
```

The `ecaz` CLI also supports profile-based corpus and benchmark commands for
`ec_hnsw`, `ec_ivf`, and `ec_diskann`; see
[`crates/ecaz-cli/README.md`](../crates/ecaz-cli/README.md).

## Methodology

See [Recall Methodology](recall-methodology.md) and
[Real Corpus Recall](RECALL_REAL_CORPUS.md) for the dataset contracts, corpus
selection rules, and reproduction instructions.
