# Task 31 M5 100k n128 nprobe Sweep

Reviewer: please review this adjacent nprobe sweep on the loaded 100k
`n128,w500` IVF surface from packet `30173`.

## Scope

This packet does not load a new surface. It reuses
`task31_m5_real100k_pqg8_n128` and sweeps `nprobe` at fixed
`rerank_width=500` to test whether the fast selected point can recover quality.

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
- Rerank width: `500`
- Swept `nprobe`: `40,48,56,64,80,96`
- Surface isolation: one-index-per-table Task 31 prefix from packet `30173`.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Results

Recall@10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 40 | 0.9760 | 0.9977 | 6.12 ms |
| 48 | 0.9820 | 0.9981 | 6.45 ms |
| 56 | 0.9860 | 0.9990 | 7.20 ms |
| 64 | 0.9890 | 0.9990 | 7.90 ms |
| 80 | 0.9960 | 0.9994 | 9.34 ms |
| 96 | 0.9980 | 0.9997 | 10.71 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 40 | 100 | 5.85 ms | 5.78 ms | 6.33 ms | 7.03 ms | 9.20 ms |
| 48 | 100 | 6.59 ms | 6.53 ms | 7.12 ms | 7.54 ms | 9.76 ms |
| 56 | 100 | 7.23 ms | 7.18 ms | 7.85 ms | 8.15 ms | 11.4 ms |
| 64 | 100 | 8.00 ms | 7.99 ms | 8.56 ms | 8.71 ms | 11.3 ms |
| 80 | 100 | 9.39 ms | 9.33 ms | 10.1 ms | 10.4 ms | 13.7 ms |
| 96 | 100 | 10.9 ms | 10.9 ms | 11.6 ms | 12.1 ms | 14.9 ms |

Backend memory sampling reported `memory_samples=0` for every latency point, so
no memory HWM claim is made.

Recall@100 for the two best recall@10 candidates:

| nprobe | recall@100 | NDCG@100 | mean q-time |
|---:|---:|---:|---:|
| 80 | 0.9639 | 0.9988 | 10.07 ms |
| 96 | 0.9676 | 0.9991 | 11.27 ms |

## Interpretation

Higher `nprobe` recovers quality while keeping much of the `n128,w500` latency
advantage. The strongest point in this sweep is `nprobe=96`: recall@10 rises to
`0.9980`, and p50 latency is `10.9 ms`.

Against the fixed 100k `n64,p48,w750` packet `30172`, `n128,p96,w500` is both
higher recall@10 and lower latency:

- `30172`: recall@10 `0.9940`, p50 `11.7 ms`, p95 `12.7 ms`
- this packet `p96`: recall@10 `0.9980`, p50 `10.9 ms`, p95 `11.6 ms`

The recall@100 result is still below ideal at `0.9676`, so the next tuning axis
should probably test rerank width on the `n128,p80/p96` region before treating
this as the selected point.

## Validation

No cargo or pgrx tests were run for this packet. Validation was packet-local
PG18 recall@10, latency, and recall@100 captures on the loaded `30173` surface.

## Next Checkpoint

Run a narrow rerank-width sweep on `task31_m5_real100k_pqg8_n128`, likely
`nprobe=80` and `96` at `rerank_width=500,750,1000`, to determine whether
recall@100 improves enough to justify extra heap rerank.

## Artifacts

- `artifacts/recall10_real100k_pqg8_n128_w500_p40_48_56_64_80_96.log`
- `artifacts/latency_real100k_pqg8_n128_w500_p40_48_56_64_80_96.log`
- `artifacts/recall100_real100k_pqg8_n128_w500_p80_96.log`
- `artifacts/manifest.md`
