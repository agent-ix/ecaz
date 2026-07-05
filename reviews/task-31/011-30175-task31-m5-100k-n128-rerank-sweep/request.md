# Task 31 M5 100k n128 Rerank Sweep

Reviewer: please review this narrow rerank-width sweep on the loaded 100k
`n128` IVF surface before selecting the next Phase B point.

## Scope

This packet reuses `task31_m5_real100k_pqg8_n128` from packet `30173` and tests
whether wider heap rerank improves quality in the promising high-probe region
from packet `30174`.

The sweep covers `nprobe=80,96` at `rerank_width=750,1000`. Packet `30174`
already contains the matching `rerank_width=500` baseline for these same probe
values.

## Surface

- Prefix: `task31_m5_real100k_pqg8_n128`
- Corpus rows: `100000`
- Query rows: `1000`
- Dimensions: `1536`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists=128`
- Rerank mode: `heap_f32`
- Swept `nprobe`: `80,96`
- Swept rerank width: `750,1000`
- Surface isolation: one-index-per-table Task 31 prefix from packet `30173`.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Results

Recall@10:

| nprobe | width | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 80 | 750 | 0.9960 | 0.9994 | 10.60 ms |
| 96 | 750 | 0.9980 | 0.9997 | 11.95 ms |
| 80 | 1000 | 0.9960 | 0.9994 | 11.72 ms |
| 96 | 1000 | 0.9980 | 0.9997 | 12.83 ms |

Recall@100:

| nprobe | width | recall@100 | NDCG@100 | mean q-time |
|---:|---:|---:|---:|---:|
| 80 | 750 | 0.9805 | 0.9992 | 11.05 ms |
| 96 | 750 | 0.9843 | 0.9995 | 12.40 ms |
| 80 | 1000 | 0.9880 | 0.9994 | 12.16 ms |
| 96 | 1000 | 0.9920 | 0.9997 | 13.41 ms |

Latency:

| nprobe | width | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 80 | 750 | 100 | 10.6 ms | 10.6 ms | 11.5 ms | 11.7 ms | 15.3 ms |
| 96 | 750 | 100 | 11.9 ms | 11.9 ms | 12.7 ms | 13.1 ms | 16.6 ms |
| 80 | 1000 | 100 | 11.7 ms | 11.6 ms | 12.9 ms | 13.2 ms | 16.6 ms |
| 96 | 1000 | 100 | 13.1 ms | 13.1 ms | 13.8 ms | 14.2 ms | 18.9 ms |

Backend memory sampling reported `memory_samples=0` for every latency point, so
no memory HWM claim is made.

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

| counter | value |
|---|---:|
| execution time | 20.366 ms |
| shared hit blocks | 8172 |
| shared read blocks | 360 |
| centroid scores | 128 |
| selected lists | 96 |
| posting pages read | 1815 |
| postings visited | 77760 |
| postings scored | 6509 |
| postings pruned by bound | 71251 |
| candidates inserted | 6509 |
| rerank rows | 1000 |
| filtered duplicates | 0 |

## Interpretation

Wider heap rerank does not improve recall@10 in the `p80/p96` region; the
recall@10 ceiling stays `0.9980` at `p96`. It does materially improve
recall@100:

- `p96,w500`: recall@100 `0.9676`, p50 `10.9 ms` from packet `30174`
- `p96,w750`: recall@100 `0.9843`, p50 `11.9 ms`
- `p96,w1000`: recall@100 `0.9920`, p50 `13.1 ms`

The `p96,w1000` point is quality-biased and slower than the fixed-scale
`n64,p48,w750` packet `30172` p50 (`13.1 ms` versus `11.7 ms`), but it improves
recall@10 from `0.9940` to `0.9980` and gives the best recall@100 seen in this
Task 31 100k M5 sweep so far.

For a balanced default-like point, `p96,w500` remains attractive. For a
quality-biased point, `p96,w1000` is the current winner, pending repeatability
checks if it is used for an optimization checkpoint.

## Validation

No cargo or pgrx tests were run for this packet. Validation was packet-local
PG18 recall@10, recall@100, latency, and one representative EXPLAIN/counter
capture on the loaded `30173` surface.

## Next Checkpoint

Use these packets to choose either:

- a balanced M5 recommendation candidate: `n128,p96,w500`
- a quality-biased candidate: `n128,p96,w1000`

Before treating either as selected, run a repeatability packet for the chosen
point per packet `30165`'s repeatability rule.

## Artifacts

- `artifacts/recall10_real100k_pqg8_n128_p80_96_w750.log`
- `artifacts/recall100_real100k_pqg8_n128_p80_96_w750.log`
- `artifacts/recall10_real100k_pqg8_n128_p80_96_w1000.log`
- `artifacts/recall100_real100k_pqg8_n128_p80_96_w1000.log`
- `artifacts/latency_real100k_pqg8_n128_p80_96_w750.log`
- `artifacts/latency_real100k_pqg8_n128_p80_96_w1000.log`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.sql`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/manifest.md`
