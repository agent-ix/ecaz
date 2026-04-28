# Task 28 IVF PQ-FastScan Bound Prune Smoke

## Scope

This packet records a bounded 100k smoke for the A7 PQ-FastScan score-bound pruning implementation in `fa4fea66`, measured at review-packet head `f47678a2`.

Fixture:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=500`
- `k=10`

## Result

| metric | prior current-head packet | bound-prune smoke |
|---|---:|---:|
| recall@10 | 0.9920 | 0.9920 |
| latency p50 | 240.7 ms | 173.1 ms |
| latency p95 | 267.2 ms | 204.9 ms |
| latency p99 | 278.9 ms | 210.5 ms |
| HWM peak | 157108 kB | 156692 kB |

Prior recall is from packet 30111. Prior latency/HWM is from packet 30113. Current raw output is stored in this packet:

- `artifacts/recall10_pqg8_100k_n128_w500_bound.log`
- `artifacts/latency_pqg8_100k_n128_w500_bound.log`

## Interpretation

At the low-latency 100k n128/nprobe48 point, A7's PQ-FastScan suffix-bound pruning preserves recall@10 and materially reduces latency in this local PG18 smoke. Memory HWM remains essentially flat.

This is still a single-point smoke, not a full A9/A10 closure matrix. It is enough to move A7 from "not done" to "implemented with positive 100k smoke evidence" and justifies continuing to the broader 100k+ IVF measurement slice.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30116-task28-ivf-pqfastscan-bound-prune-smoke/artifacts/recall10_pqg8_100k_n128_w500_bound.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30116-task28-ivf-pqfastscan-bound-prune-smoke/artifacts/latency_pqg8_100k_n128_w500_bound.log`

## Next

Use this nprobe48 point as the default low-latency PQ-FastScan IVF operating point for the next A9/A10 slice. The next measurement should add recall@100 and the remaining cache/build-size fields rather than reopening HNSW comparison work.
