# Task 28 IVF 990k Rerank Width Latency

## Scope

This packet checks whether reducing heap rerank width improves latency at the current 990k balanced point from packet 30133.

Fixture:

- prefix: `task28_ivf_pqg990k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `nprobe=40`
- `rerank=heap_f32`
- cache state: warm local PG18; no OS or Postgres cache drop
- iterations: 50 per point

## Result

| rerank_width | latency p50 | latency p95 | latency p99 | mean | RSS peak KB |
|---:|---:|---:|---:|---:|---:|
| 250 | 857.5 ms | 974.1 ms | 1014.1 ms | 856.6 ms | 166216 |
| 500 | 891.6 ms | 1003.0 ms | 1043.6 ms | 896.2 ms | 166376 |

## Interpretation

At nprobe 40 on the 990k surface, lowering `rerank_width` from 500 to 250 improves this matched 50-iteration latency sample by about 34 ms p50 and 29 ms p95. Memory is effectively unchanged.

This is not enough to recommend width 250 as the new balanced point because the 990k recall check for width 250 did not complete in a reasonable local window. Keep `nprobe=40, rerank_width=500` as the recall-backed balanced point until recall for narrower widths can use a cheaper exact-truth cache.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 50 --sweep 40 --rerank-width 250 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30136-task28-ivf-990k-rerank-width/artifacts/latency_pqg8_990k_nprobe40_w250_i50.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 50 --sweep 40 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30136-task28-ivf-990k-rerank-width/artifacts/latency_pqg8_990k_nprobe40_w500_i50.log`

## Notes

A width-250 recall@10 attempt was stopped after roughly 21 minutes while the harness was still CPU-bound in the exact-truth setup. No recall claim is made in this packet.

## Artifacts

- `artifacts/latency_pqg8_990k_nprobe40_w250_i50.log`
- `artifacts/latency_pqg8_990k_nprobe40_w500_i50.log`
- `artifacts/manifest.md`
