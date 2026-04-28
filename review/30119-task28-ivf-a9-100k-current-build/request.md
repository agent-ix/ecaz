# Task 28 IVF A9 100k Current Build

## Scope

This packet records a fresh current-head rebuild and selected-point measurement for the 100k IVF PQ-FastScan surface after A7 score-bound pruning.

Fixture:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=500`
- PG18 local database

## Result

| metric | value |
|---|---:|
| build time | 216788.531 ms |
| index size | 19,791,872 bytes |
| recall@10 | 0.9920 |
| NDCG@10 | 0.9997 |
| recall@100 | 0.9552 |
| NDCG@100 | 0.9983 |
| latency p50 | 173.4 ms |
| latency p95 | 225.4 ms |
| latency p99 | 242.9 ms |
| memory HWM | 157108 kB |

Cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop.

Raw logs are stored under `artifacts/`:

- `build_pqg8_100k_n128_w500_current.log`
- `recall10_pqg8_100k_n128_w500_fresh.log`
- `recall100_pqg8_100k_n128_w500_fresh.log`
- `latency_pqg8_100k_n128_w500_fresh.log`

## Interpretation

This closes the fresh-current-head 100k IVF side of A9 for the selected low-latency operating point. It replaces the prior need to cite packet 30092 for build time on this surface and confirms the A7 pruning result survives a fresh rebuild.

The result keeps the same recommendation from packets 30116 and 30117: `nlists=128`, `nprobe=48`, PQ-FastScan group size 8, and heap rerank width 500 is the current 100k IVF selected point.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30119-task28-ivf-a9-100k-current-build/artifacts/build_pqg8_100k_n128_w500_current.sql --raw --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/build_pqg8_100k_n128_w500_current.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/recall10_pqg8_100k_n128_w500_fresh.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/recall100_pqg8_100k_n128_w500_fresh.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/latency_pqg8_100k_n128_w500_fresh.log`

## Remaining A9 Gap

This packet intentionally does not run the matched HNSW comparison or 1M matrix. The remaining A9 gap is the larger comparison packet; this packet gives that future packet a fresh IVF anchor without blocking current IVF implementation momentum.
