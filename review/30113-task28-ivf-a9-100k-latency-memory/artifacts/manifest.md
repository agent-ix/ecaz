# Artifact Manifest

## latency_pqg8_100k_n128_w500_memory.log

- head SHA: `1b50dace`
- packet/topic: `30113-task28-ivf-a9-100k-latency-memory`
- lane: Task 28 IVF A9 100k latency memory HWM
- fixture: existing isolated `task28_ivf_pqg100k_g8_n128` 100k corpus, 100 query table, 100 latency iterations per sweep value
- storage format / quantizer: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48,56,64 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30113-task28-ivf-a9-100k-latency-memory/artifacts/latency_pqg8_100k_n128_w500_memory.log`
- timestamp: `2026-04-28T12:13:45-07:00`
- isolated/shared surface: isolated one-index-per-table surface
- key cited result lines:
  - `nprobe=48`: `count=100`, `mean=242.8 ms`, `p50=240.7 ms`, `p95=267.2 ms`, `p99=278.9 ms`, `rss_peak_kb=157108`, `hwm_peak_kb=157108`, `memory_samples=908`
  - `nprobe=56`: `count=100`, `mean=277.1 ms`, `p50=275.2 ms`, `p95=308.1 ms`, `p99=329.4 ms`, `rss_peak_kb=156812`, `hwm_peak_kb=156812`, `memory_samples=1042`
  - `nprobe=64`: `count=100`, `mean=304.3 ms`, `p50=304.2 ms`, `p95=337.3 ms`, `p99=343.9 ms`, `rss_peak_kb=159300`, `hwm_peak_kb=159300`, `memory_samples=1139`
