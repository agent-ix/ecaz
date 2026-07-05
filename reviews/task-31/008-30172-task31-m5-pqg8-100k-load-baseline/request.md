# Task 31 M5 PQ-FastScan g8 100k Load Baseline

Reviewer: please review this 100k real M5 IVF baseline packet before Phase B
bottleneck selection.

## Scope

This packet loads the real DBPedia-derived 100k corpus staged in packet `30167`
and captures the fixed PQ-FastScan group-size-8 baseline from packet `30165`.

It keeps the same `nlists=64`, `nprobe=48`, and `rerank_width=750` settings as
the 10k, 25k, and 50k packets so the scale progression remains directly
comparable.

## Surface

- Prefix: `task31_m5_real100k_pqg8_n64`
- Corpus rows: `100000`
- Query rows: `1000`
- Dimensions: `1536`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists=64`
- `nprobe=48`
- Rerank mode: `heap_f32`
- Rerank width: `750`
- Surface isolation: one Task 31 corpus table with one `ec_ivf` index plus the
  btree primary key.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Load / Build

The load used `--allow-manifest-mismatch` because the staged corpus manifest
prefix is `ec_hnsw_real_100k` while the Task 31 load prefix is
`task31_m5_real100k_pqg8_n64`. The corpus and query hashes matched the staged
manifest.

| step | result |
|---|---:|
| corpus copy | `19.17s` |
| ecvector encode | `7.89s` |
| query copy | `174.47ms` |
| IVF index build | `9.55s` |
| full load/build | `42.92s` |

## Results

Recall@10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 48 | 0.9940 | 0.9996 | 11.72 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 100 | 11.7 ms | 11.7 ms | 12.7 ms | 13.3 ms | 16.4 ms |

Backend memory sampling reported `memory_samples=0`, so no memory HWM claim is
made for this 100k surface.

Storage:

| field | value |
|---|---:|
| table total | 1.6 GiB |
| all indexes | 22.9 MiB |
| IVF index | 18.6 MiB |
| IVF index per row | 195.4 B |

Representative EXPLAIN/counters:

| counter | value |
|---|---:|
| execution time | 19.267 ms |
| shared hit blocks | 3656 |
| shared read blocks | 3122 |
| centroid scores | 64 |
| selected lists | 48 |
| posting pages read | 1753 |
| postings visited | 76022 |
| postings scored | 5072 |
| postings pruned by bound | 70950 |
| candidates inserted | 5072 |
| rerank rows | 750 |
| filtered duplicates | 0 |

## Interpretation

At 100k, the fixed `nprobe=48` / `rerank_width=750` setting is still fast on
the local M5 PG18 surface, but recall@10 drops to `0.9940`. This is the first
scale point in the 10k/25k/50k/100k progression where the fixed setting no
longer preserves near-perfect recall for the sampled queries.

The counter shape suggests the same Phase B pressure as the smaller surfaces:
many postings are visited, most are pruned by the PQ-FastScan bound, and only
`5072` candidates reach scoring before the fixed `750` heap rerank. Treat this
as local M5 development evidence only, not a product-class claim.

## Validation

No cargo or pgrx tests were run for this packet. Validation was the
packet-local PG18 load, recall, latency, storage, and EXPLAIN/counter captures.

## Next Checkpoint

Use the 10k/25k/50k/100k baseline packets to choose the first Phase B
optimization or tuning axis. The 100k point is now a useful candidate for an
`nprobe` and/or `rerank_width` quality/latency sweep because the fixed baseline
recall dropped to `0.9940`.

## Artifacts

- `artifacts/load_real100k_pqg8_n64_w750_allow_manifest_mismatch.log`
- `artifacts/recall10_real100k_pqg8_n64_p48_w750.log`
- `artifacts/truth_real100k_k10.json`
- `artifacts/latency_real100k_pqg8_n64_p48_w750.log`
- `artifacts/storage_real100k_pqg8_n64.log`
- `artifacts/explain_real100k_pqg8_n64_p48_w750.sql`
- `artifacts/explain_real100k_pqg8_n64_p48_w750.log`
- `artifacts/manifest.md`
