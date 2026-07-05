# Task 31 M5 PQ-FastScan g8 25k Load Baseline

Reviewer: please review this 25k real M5 IVF baseline packet before larger
Task 31 surfaces are loaded.

## Scope

This packet loads the real DBPedia-derived 25k corpus staged in packet `30167`
and captures the fixed PQ-FastScan group-size-8 baseline from packet `30165`.

It intentionally keeps the same `nlists=64`, `nprobe=48`, and
`rerank_width=750` settings as the 10k packet so the 10k and 25k measurements
are directly comparable.

## Surface

- Prefix: `task31_m5_real25k_pqg8_n64`
- Corpus rows: `25000`
- Query rows: `500`
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
prefix is `ec_hnsw_real_25k` while the Task 31 load prefix is
`task31_m5_real25k_pqg8_n64`. The corpus and query hashes matched the staged
manifest.

| step | result |
|---|---:|
| corpus copy | `3.79s` |
| ecvector encode | `765.04ms` |
| query copy | `74.30ms` |
| IVF index build | `5.16s` |
| full load/build | `11.33s` |

## Results

Recall@10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 48 | 0.9990 | 1.0000 | 4.65 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 100 | 4.87 ms | 4.78 ms | 5.53 ms | 6.12 ms | 8.40 ms |

Backend memory sampling reported `memory_samples=0`, so no memory HWM claim is
made for this still-fast 25k surface.

Storage:

| field | value |
|---|---:|
| table total | 398.4 MiB |
| all indexes | 6.4 MiB |
| IVF index | 5.3 MiB |
| IVF index per row | 223.2 B |

Representative EXPLAIN/counters:

| counter | value |
|---|---:|
| execution time | 8.648 ms |
| shared hit blocks | 5141 |
| shared read blocks | 309 |
| centroid scores | 64 |
| selected lists | 48 |
| posting pages read | 424 |
| postings visited | 17547 |
| postings scored | 2705 |
| postings pruned by bound | 14842 |
| candidates inserted | 2705 |
| rerank rows | 750 |
| filtered duplicates | 0 |

## Interpretation

The 25k point remains fast on the local M5 PG18 surface, with recall@10 just
below perfect at `0.9990` and p50 latency under 5 ms. Compared with the 10k
packet, the representative EXPLAIN shows postings visited increasing from
`7638` to `17547`, while fully scored postings increase more modestly from
`2039` to `2705`.

Treat this as local M5 development evidence only, not a product-class claim.
Larger 50k/100k surfaces are still needed before selecting a Phase B
bottleneck.

## Validation

No cargo or pgrx tests were run for this packet. Validation was the
packet-local PG18 load, recall, latency, storage, and EXPLAIN/counter captures.

## Next Checkpoint

Continue the Task 31 scale progression with the larger staged real-corpus
profiles, keeping the same fixed reloptions until the scale measurements show a
clear first bottleneck.

## Artifacts

- `artifacts/load_real25k_pqg8_n64_w750_allow_manifest_mismatch.log`
- `artifacts/recall10_real25k_pqg8_n64_p48_w750.log`
- `artifacts/truth_real25k_k10.json`
- `artifacts/latency_real25k_pqg8_n64_p48_w750.log`
- `artifacts/storage_real25k_pqg8_n64.log`
- `artifacts/explain_real25k_pqg8_n64_p48_w750.sql`
- `artifacts/explain_real25k_pqg8_n64_p48_w750.log`
- `artifacts/manifest.md`
