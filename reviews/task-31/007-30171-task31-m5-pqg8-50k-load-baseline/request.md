# Task 31 M5 PQ-FastScan g8 50k Load Baseline

Reviewer: please review this 50k real M5 IVF baseline packet before the 100k
surface is loaded.

## Scope

This packet loads the real DBPedia-derived 50k corpus staged in packet `30167`
and captures the fixed PQ-FastScan group-size-8 baseline from packet `30165`.

It keeps the same `nlists=64`, `nprobe=48`, and `rerank_width=750` settings as
the 10k and 25k packets so the scale progression remains directly comparable.

## Surface

- Prefix: `task31_m5_real50k_pqg8_n64`
- Corpus rows: `50000`
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
prefix is `ec_hnsw_real_50k` while the Task 31 load prefix is
`task31_m5_real50k_pqg8_n64`. The corpus and query hashes matched the staged
manifest.

| step | result |
|---|---:|
| corpus copy | `7.54s` |
| ecvector encode | `1.58s` |
| query copy | `148.28ms` |
| IVF index build | `6.59s` |
| full load/build | `18.86s` |

## Results

Recall@10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 48 | 1.0000 | 1.0000 | 6.82 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 100 | 6.69 ms | 6.63 ms | 7.24 ms | 7.73 ms | 10.6 ms |

Backend memory sampling reported `memory_samples=0`, so no memory HWM claim is
made for this fast 50k surface.

Storage:

| field | value |
|---|---:|
| table total | 796.6 MiB |
| all indexes | 11.9 MiB |
| IVF index | 9.7 MiB |
| IVF index per row | 203.8 B |

Representative EXPLAIN/counters:

| counter | value |
|---|---:|
| execution time | 12.542 ms |
| shared hit blocks | 3926 |
| shared read blocks | 1909 |
| centroid scores | 64 |
| selected lists | 48 |
| posting pages read | 810 |
| postings visited | 34639 |
| postings scored | 3895 |
| postings pruned by bound | 30744 |
| candidates inserted | 3895 |
| rerank rows | 750 |
| filtered duplicates | 0 |

## Interpretation

The 50k scale point remains within single-digit millisecond p50 latency on the
local M5 PG18 surface while preserving recall@10 `1.0000` for the sampled
queries. The counter shape continues the 10k and 25k trend: postings visited
scale up substantially, but bounded PQ-FastScan scoring keeps the fully scored
set much smaller before heap rerank.

Treat this as local M5 development evidence only, not a product-class claim.
The next scale point should be 100k before Phase B bottleneck selection.

## Validation

No cargo or pgrx tests were run for this packet. Validation was the
packet-local PG18 load, recall, latency, storage, and EXPLAIN/counter captures.

## Next Checkpoint

Create `30172-task31-m5-pqg8-100k-load-baseline` and load the staged 100k TSVs
with the same fixed reloptions:

- `profile=ec_ivf`
- `storage_format=pq_fastscan`
- `pq_group_size=8`
- `nlists=64`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=750`

Use `--allow-manifest-mismatch` unless the loader workflow is improved.

## Artifacts

- `artifacts/load_real50k_pqg8_n64_w750_allow_manifest_mismatch.log`
- `artifacts/recall10_real50k_pqg8_n64_p48_w750.log`
- `artifacts/truth_real50k_k10.json`
- `artifacts/latency_real50k_pqg8_n64_p48_w750.log`
- `artifacts/storage_real50k_pqg8_n64.log`
- `artifacts/explain_real50k_pqg8_n64_p48_w750.sql`
- `artifacts/explain_real50k_pqg8_n64_p48_w750.log`
- `artifacts/manifest.md`
