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

### Apple-Silicon M5 Inventory

This section is a benchmark inventory, not a benchmark narrative.
Every populated row below is backed by a packet-local measurement.
`--` means the benchmark slot belongs in the M5 roadmap but no checked-in,
packet-local number exists yet.

#### IVF M5

| Family | Benchmark lane | Fixture / configuration | Result | Source |
| --- | --- | --- | --- | --- |
| `ec_ivf` | balanced quality point | 100K, `pq_fastscan`, `pq_group_size=8`, `nlists=128`, `nprobe=96`, `rerank_width=500` | recall@100 `0.9676`, p50 `10.7 ms`, p95 `11.6 ms`, p99 `12.1 ms` | `review/30203-task31-current-m5-candidate-decision/` |
| `ec_ivf` | quality point | 100K, `pq_fastscan`, `pq_group_size=8`, `nlists=128`, `nprobe=96`, `rerank_width=1000` | recall@100 `0.9920`, mean q-time `12.38 ms`, p50 `12.1 ms`, p95 `13.0 ms`, p99 `13.7 ms` | `review/30203-task31-current-m5-candidate-decision/` |
| `ec_ivf` | 10K baseline | -- | -- | -- |
| `ec_ivf` | 25K baseline | -- | -- | -- |
| `ec_ivf` | 50K baseline | -- | -- | -- |
| `ec_ivf` | 990K directional point | -- | -- | -- |

#### DiskANN M5

| Family | Benchmark lane | Fixture / configuration | Result | Source |
| --- | --- | --- | --- | --- |
| `ec_diskann` | build A/B, scalar vs NEON | real10K, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | scalar `32.61 s`; NEON mean `6.81 s`; recall@10 identical at `0.9965 / 0.9970 / 0.9975` for `L=64/200/800` | `review/30208-task29-diskann-m5-build-neon-followup/` |
| `ec_diskann` | default rerank NEON A/B | real10K, default `rerank_budget=64`, `L=64/200/800`, warm cache | p50 scalar/neon `1.98/1.93 ms`, `2.20/2.15 ms`, `2.76/2.70 ms`; recall@10 identical at `0.9965 / 0.9970 / 0.9975` | `review/30204-task29-diskann-m5-neon-rerank/` |
| `ec_diskann` | kernel-stress rerank NEON A/B | real10K_w800, `rerank_budget=800`, `L=800`, warm cache | pass-avg p50 scalar/neon `16.3/15.2 ms`; p95 `17.15/15.85 ms`; p99 `18.9/16.7 ms`; recall@10 `1.0000` | `review/30204-task29-diskann-m5-neon-rerank/` |
| `ec_diskann` | heap-TID rerank fetch A/B | real10K_w800, post-NEON, `rerank_budget=800`, `L=800`, warm cache | pass-avg p50 pre/post `15.5/14.8 ms`; p95 `16.15/15.45 ms`; p99 `17.9/16.8 ms`; recall@10 `1.0000` | `review/30205-task29-diskann-m5-rerank-heap-order/` |
| `ec_diskann` | heap-block prefetch A/B | real10K_w800, post-heap-order, `rerank_budget=800`, `L=800`, warm cache | pass-avg p50 pre/trial `14.8/15.0 ms`; p95 `15.45/15.6 ms`; p99 `16.8/16.85 ms`; recall@10 `1.0000` | `review/30206-task29-diskann-m5-rerank-prefetch/` |
| `ec_diskann` | cold-cache prefetch A/B | real100K, `rerank_budget=800`, `L=800`, heap `12.6x shared_buffers`, first-pass cold start | p50 pre/prefetch `506.2/406.8 ms`; p95 `633.2/426.8 ms`; p99 `676.9/434.3 ms`; recall@10 `0.9978` | `review/30209-task29-diskann-m5-cold-cache-100k/` |
| `ec_diskann` | full post-M5 cross-engine sweep | final M5 code state, Task 29d search-list sweep `64/128/200/400/800` | -- | -- |
| `ec_diskann` | async-overlap prefetch trial | cold-cache rerank lane | -- | -- |
| `ec_diskann` | same-page-run grouping trial | cold-cache rerank lane | -- | -- |

#### HNSW M5

| Family | Benchmark lane | Fixture / configuration | Result | Source |
| --- | --- | --- | --- | --- |
| `ec_hnsw` | 50K reference refresh | current default `ConcurrentDsm`, worker sweep | -- | -- |
| `ec_hnsw` | larger-corpus reference refresh | current default `ConcurrentDsm`, worker sweep | -- | -- |
| `ec_hnsw` | build worker curve | `1/2/4/8` requested workers with PG18 headroom recorded | -- | -- |

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

### SQL and corpus benchmarks

Requires PostgreSQL with the extension installed:

```bash
make bench-sql-latency
make bench-storage
make bench-recall-sql
```

The supported repeatable operator surface is the `ecaz` CLI. It prepares and
loads corpora, builds profile-specific indexes, runs recall/latency/storage
benchmarks, compares external engines, and writes packet-local logs:

```bash
ecaz corpus prepare --profile ec_hnsw_real_10k --parquet /path/to/parquet --output-dir /path/to/staged
ecaz corpus load --prefix ec_hnsw_real_10k --corpus-file /path/to/staged/ec_hnsw_real_10k_corpus.tsv --queries-file /path/to/staged/ec_hnsw_real_10k_queries.tsv --profile ec_hnsw --log-file review/example/artifacts/load.log
ecaz bench recall --prefix ec_hnsw_real_10k --profile ec_hnsw --log-file review/example/artifacts/recall.log
ecaz bench latency --prefix ec_hnsw_real_10k --profile ec_hnsw --log-file review/example/artifacts/latency.log
```

See the [Operator CLI README](../crates/ecaz-cli/README.md) for all command
groups and profile behavior.

## Methodology

See [Recall Methodology](recall-methodology.md) and
[Real Corpus Recall](RECALL_REAL_CORPUS.md) for the dataset contracts, corpus
selection rules, and reproduction instructions.
