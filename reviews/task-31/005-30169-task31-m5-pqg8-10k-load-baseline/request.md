# Task 31 M5 PQ-FastScan g8 10k Load Baseline

Reviewer: please review this first real M5 IVF baseline packet before the 25k
and 100k surfaces are loaded.

## Scope

This packet loads the real DBPedia-derived 10k corpus staged in packet `30167`
and captures the fixed 10k PQ-FastScan group-size-8 baseline from packet
`30165`.

It intentionally stops at the 10k surface. It does not load 25k or 100k.

## Surface

- Prefix: `task31_m5_real10k_pqg8_n64`
- Corpus rows: `10000`
- Query rows: `200`
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

The first load attempt intentionally failed before creating the surface because
the staged corpus manifest prefix is `ec_hnsw_real_10k` while the Task 31 load
prefix is `task31_m5_real10k_pqg8_n64`. The corpus and query hashes matched.

The successful load used `--allow-manifest-mismatch` to keep the Task
31-specific table prefix:

| step | result |
|---|---:|
| corpus copy | `1.95s` |
| ecvector encode | `646.17ms` |
| query copy | `32.65ms` |
| IVF index build | `4.29s` |
| full load/build | `7.55s` |

## Results

Recall@10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 48 | 1.0000 | 1.0000 | 3.05 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 100 | 3.06 ms | 2.96 ms | 3.51 ms | 4.23 ms | 7.20 ms |

The 1ms memory-sampling retry reported p50 `3.00 ms`, p95 `3.88 ms`, and p99
`4.51 ms`, but still captured `memory_samples=0`. No memory HWM claim is made
for this very fast 10k surface.

Storage:

| field | value |
|---|---:|
| table total | 159.4 MiB |
| all indexes | 3.1 MiB |
| IVF index | 2.6 MiB |
| IVF index per row | 277.7 B |

Representative EXPLAIN/counters:

| counter | value |
|---|---:|
| execution time | 6.337 ms |
| shared hit blocks | 4461 |
| shared read blocks | 0 |
| centroid scores | 64 |
| selected lists | 48 |
| posting pages read | 198 |
| postings visited | 7638 |
| postings scored | 2039 |
| postings pruned by bound | 5599 |
| candidates inserted | 2039 |
| rerank rows | 750 |
| filtered duplicates | 0 |

## Interpretation

The real 10k M5 baseline is now dramatically faster than the older Task 28
local x86 packet `30137` for the same nominal PQ-FastScan g8 10k point
(`p50=2.96 ms` here versus `77.3 ms` there). Treat this as local M5 development
evidence only, not a product-class claim.

The EXPLAIN counters still show the same qualitative shape as Task 28: many
postings are visited, fewer are fully scored, and the bound prunes a large
fraction before heap rerank. At this 10k scale, wall time is too low to choose a
first optimization target by itself; 25k and 100k must be measured before
Phase B bottleneck selection.

## Validation

No cargo or pgrx tests were run for this packet. Validation was the
packet-local PG18 load, recall, latency, storage, and EXPLAIN/counter captures.

## Next Checkpoint

Create `30170-task31-m5-pqg8-25k-load-baseline` and load the staged 25k TSVs
with the same 10k/25k reloptions:

- `profile=ec_ivf`
- `storage_format=pq_fastscan`
- `pq_group_size=8`
- `nlists=64`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=750`

Use `--allow-manifest-mismatch` or adjust the loader workflow so Task
31-specific prefixes can cite staged corpus manifests without weakening hash
verification.

## Artifacts

- `artifacts/load_real10k_pqg8_n64_w750.log`
- `artifacts/load_real10k_pqg8_n64_w750_allow_manifest_mismatch.log`
- `artifacts/recall10_real10k_pqg8_n64_p48_w750.log`
- `artifacts/truth_real10k_k10.json`
- `artifacts/latency_real10k_pqg8_n64_p48_w750.log`
- `artifacts/latency_real10k_pqg8_n64_p48_w750_mem1ms.log`
- `artifacts/storage_real10k_pqg8_n64.log`
- `artifacts/explain_real10k_pqg8_n64_p48_w750.sql`
- `artifacts/explain_real10k_pqg8_n64_p48_w750.log`
- `artifacts/manifest.md`
