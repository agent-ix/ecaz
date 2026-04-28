# Task 28 IVF A9 100k Latency Memory HWM

## Scope

This packet reruns the 100k IVF n128 latency frontier with `--sample-backend-memory` after packet 30112 added benchmark support for backend RSS/HWM columns.

Fixture:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `rerank=heap_f32`
- `rerank_width=500`
- `k=10`
- 100 latency iterations per nprobe

## Result

| nprobe | p50 | p95 | p99 | RSS peak | HWM peak | memory samples |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 240.7 ms | 267.2 ms | 278.9 ms | 157108 kB | 157108 kB | 908 |
| 56 | 275.2 ms | 308.1 ms | 329.4 ms | 156812 kB | 156812 kB | 1042 |
| 64 | 304.2 ms | 337.3 ms | 343.9 ms | 159300 kB | 159300 kB | 1139 |

Raw output is in `artifacts/latency_pqg8_100k_n128_w500_memory.log`.

## Interpretation

This fills the scan-memory part of the A9 100k IVF n128 packet for the current-head low-latency frontier. The memory HWM is stable across the measured nprobe range, roughly 157-159 MB in this local PG18 run.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48,56,64 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30113-task28-ivf-a9-100k-latency-memory/artifacts/latency_pqg8_100k_n128_w500_memory.log`

## Next

Use the same flag for A10 variant comparisons. This packet only covers the current n128 PQ-FastScan operating point, not the full quantizer comparison matrix.
