# Task 28 IVF 990k Lower Nprobe Frontier

## Scope

This packet uses the existing isolated 990k IVF surface from packet 30130 and sweeps lower scan-time `nprobe` values at fixed `rerank_width=500`.

Fixture:

- prefix: `task28_ivf_pqg990k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `rerank=heap_f32`
- `rerank_width=500`
- 100 latency iterations per point
- 100-query exact recall cap

## Result

| nprobe | recall@10 | NDCG@10 | recall mean q-time | latency p50 | latency p95 | latency p99 | HWM KB |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 16 | 0.9380 | 0.9959 | 448.30 ms | 432.3 ms | 522.8 ms | 575.0 ms | 157340 |
| 24 | 0.9640 | 0.9976 | 590.08 ms | 580.5 ms | 678.6 ms | 730.7 ms | 166340 |
| 32 | 0.9750 | 0.9984 | 741.11 ms | 740.1 ms | 833.8 ms | 876.6 ms | 162588 |
| 40 | 0.9810 | 0.9987 | 897.16 ms | 884.2 ms | 994.8 ms | 1036.2 ms | 162636 |
| 48 | 0.9860 | 0.9990 | 1042.06 ms | 1042.8 ms | 1179.9 ms | 1229.2 ms | 162664 |

## Interpretation

The selected nprobe 48 point from packet 30130 is not the best latency frontier point for the 990k surface.

Two better candidates emerge:

- `nprobe=32`: p50 stays around 740 ms and recall@10 remains 0.9750.
- `nprobe=40`: p50 stays under 900 ms and recall@10 rises to 0.9810, with p95 just under 1s.

The marginal gain from `nprobe=40` to `nprobe=48` is only +0.005 recall@10, while p50 increases by about 159 ms and p95 by about 185 ms.

## Next

Run a narrower recall@100 follow-up for `nprobe in {32,40}`. If recall@100 holds, carry `nprobe=40` as the 990k balanced point and `nprobe=32` as the lower-latency point.

Also add scan score-volume counters in the next code slice so latency changes can be tied to postings scored and candidates reranked, not just wall time.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 16,24,32,40,48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30132-task28-ivf-990k-lower-nprobe-latency/artifacts/latency_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 16,24,32,40,48 --rerank-width 500 --force-index --log-output review/30132-task28-ivf-990k-lower-nprobe-latency/artifacts/recall10_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log`

## Artifacts

- `artifacts/latency_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log`
- `artifacts/recall10_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log`
- `artifacts/manifest.md`
