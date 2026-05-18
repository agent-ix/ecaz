# Task 31 M5 PQ-FastScan g8 100k n128 w500 Baseline

Reviewer: please review this selected 100k M5 IVF point before Phase B tuning
selection.

## Scope

This packet loads the Task 31 plan-selected 100k surface:
`nlists=128,nprobe=48,rerank_width=500`. It is directly comparable with the
fixed-scale 100k packet `30172`, which used `nlists=64,nprobe=48,w750`.

## Surface

- Prefix: `task31_m5_real100k_pqg8_n128`
- Corpus rows: `100000`
- Query rows: `1000`
- Dimensions: `1536`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists=128`
- `nprobe=48`
- Rerank mode: `heap_f32`
- Rerank width: `500`
- Surface isolation: one Task 31 corpus table with one `ec_ivf` index plus the
  btree primary key.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Load / Build

The load used `--allow-manifest-mismatch` because the staged corpus manifest
prefix is `ec_hnsw_real_100k` while the Task 31 load prefix is
`task31_m5_real100k_pqg8_n128`. The corpus and query hashes matched the staged
manifest.

| step | result |
|---|---:|
| corpus copy | `15.02s` |
| ecvector encode | `5.89s` |
| query copy | `183.48ms` |
| IVF index build | `18.69s` |
| full load/build | `45.76s` |

## Results

Recall:

| k | nprobe | recall | NDCG | mean q-time |
|---:|---:|---:|---:|---:|
| 10 | 48 | 0.9820 | 0.9981 | 6.72 ms |
| 100 | 48 | 0.9436 | 0.9970 | 7.19 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 100 | 6.79 ms | 6.80 ms | 7.37 ms | 7.63 ms | 9.87 ms |

Backend memory sampling reported `memory_samples=0`, so no memory HWM claim is
made.

Storage:

| field | value |
|---|---:|
| table total | 1.6 GiB |
| all indexes | 23.7 MiB |
| IVF index | 19.4 MiB |
| IVF index per row | 202.9 B |

Representative EXPLAIN/counters:

| counter | value |
|---|---:|
| execution time | 11.250 ms |
| shared hit blocks | 2672 |
| shared read blocks | 1862 |
| centroid scores | 128 |
| selected lists | 48 |
| posting pages read | 817 |
| postings visited | 34896 |
| postings scored | 2748 |
| postings pruned by bound | 32148 |
| candidates inserted | 2748 |
| rerank rows | 500 |
| filtered duplicates | 0 |

## Interpretation

Compared with the `30172` 100k `n64,p48,w750` fixed-scale baseline, this
`n128,p48,w500` point is much faster but loses recall:

- `30172`: recall@10 `0.9940`, p50 `11.7 ms`
- this packet: recall@10 `0.9820`, p50 `6.80 ms`

The counters show why latency improves: visited postings drop from `76022` to
`34896`, scored postings drop from `5072` to `2748`, and heap rerank rows drop
from `750` to `500`. This is useful as a fast point, but not as a quality
replacement without more tuning.

## Validation

No cargo or pgrx tests were run for this packet. Validation was the
packet-local PG18 load, recall@10, recall@100, latency, storage, and
EXPLAIN/counter captures.

## Next Checkpoint

Use the loaded `task31_m5_real100k_pqg8_n128` surface for an adjacent nprobe
sweep at `rerank_width=500` to see whether higher probes recover quality while
retaining the latency advantage.

## Artifacts

- `artifacts/load_real100k_pqg8_n128_w500_allow_manifest_mismatch.log`
- `artifacts/recall10_real100k_pqg8_n128_p48_w500.log`
- `artifacts/truth_real100k_n128_k10.json`
- `artifacts/recall100_real100k_pqg8_n128_p48_w500.log`
- `artifacts/truth_real100k_n128_k100.json`
- `artifacts/latency_real100k_pqg8_n128_p48_w500.log`
- `artifacts/storage_real100k_pqg8_n128.log`
- `artifacts/explain_real100k_pqg8_n128_p48_w500.sql`
- `artifacts/explain_real100k_pqg8_n128_p48_w500.log`
- `artifacts/manifest.md`
