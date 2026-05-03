# Task 31 M5 100k Candidate Repeatability

Reviewer: please review this narrow repeatability packet for the two candidate
100k `n128,p96` points identified by packets `30174` and `30175`.

## Scope

This packet reuses the loaded `task31_m5_real100k_pqg8_n128` surface and reruns
latency plus recall for:

- balanced candidate: `nprobe=96,rerank_width=500`
- quality-biased candidate: `nprobe=96,rerank_width=1000`

It does not load a new surface.

## Surface

- Prefix: `task31_m5_real100k_pqg8_n128`
- Corpus rows: `100000`
- Query rows: `1000`
- Dimensions: `1536`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists=128`
- `nprobe=96`
- Rerank mode: `heap_f32`
- Rerank widths: `500`, `1000`
- Surface isolation: one-index-per-table Task 31 prefix from packet `30173`.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Results

Repeat latency:

| width | count | mean | p50 | p95 | p99 | max |
|---:|---:|---:|---:|---:|---:|---:|
| 500 | 100 | 10.8 ms | 10.8 ms | 11.4 ms | 11.9 ms | 15.5 ms |
| 1000 | 100 | 13.0 ms | 12.9 ms | 13.9 ms | 14.3 ms | 18.7 ms |

Backend memory sampling reported `memory_samples=0` for both latency points, so
no memory HWM claim is made.

Repeat recall:

| width | k | recall | NDCG | mean q-time |
|---:|---:|---:|---:|---:|
| 500 | 10 | 0.9980 | 0.9997 | 11.00 ms |
| 500 | 100 | 0.9676 | 0.9991 | 11.41 ms |
| 1000 | 10 | 0.9980 | 0.9997 | 12.80 ms |
| 1000 | 100 | 0.9920 | 0.9997 | 13.78 ms |

## Interpretation

The repeat run preserved the main distinction from packets `30174` and `30175`:

- `p96,w500` remains the balanced candidate: recall@10 `0.9980`, repeat p50
  `10.8 ms`, but recall@100 stays `0.9676`.
- `p96,w1000` remains the quality-biased candidate: recall@10 `0.9980`,
  recall@100 `0.9920`, repeat p50 `12.9 ms`.

Compared with the fixed-scale 100k packet `30172` (`n64,p48,w750`, recall@10
`0.9940`, p50 `11.7 ms`), the repeat run supports:

- balanced candidate improves recall@10 and latency
- quality candidate improves recall@10 and recall@100, with higher latency than
  the fixed-scale point

## Validation

No cargo or pgrx tests were run for this packet. Validation was packet-local
PG18 repeat latency and repeat recall captures on the loaded `30173` surface.

## Next Checkpoint

Use `p96,w500` as the balanced candidate and `p96,w1000` as the quality-biased
candidate in the Phase B decision packet. If the next work is implementation
rather than tuning, start from the counter shape: large posting visits with
most rows pruned by PQ-FastScan bound before rerank.

## Artifacts

- `artifacts/latency_repeat_real100k_pqg8_n128_p96_w500.log`
- `artifacts/latency_repeat_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/recall10_repeat_real100k_pqg8_n128_p96_w500.log`
- `artifacts/recall100_repeat_real100k_pqg8_n128_p96_w500.log`
- `artifacts/recall10_repeat_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/recall100_repeat_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/manifest.md`
