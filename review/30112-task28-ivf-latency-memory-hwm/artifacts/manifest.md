# Artifact Manifest

## latency_memory_smoke.log

- head SHA: `306b31b5`
- packet/topic: `30112-task28-ivf-latency-memory-hwm`
- lane: Task 28 IVF latency memory-HWM benchmark support
- fixture: existing isolated `task28_ivf_pqg100k_g8_n128` 100k corpus, 100 query table, 2 latency iterations
- storage format / quantizer: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 2 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30112-task28-ivf-latency-memory-hwm/artifacts/latency_memory_smoke.log`
- timestamp: `2026-04-28T12:11:18-07:00`
- isolated/shared surface: isolated one-index-per-table surface
- key cited result lines:
  - `nprobe=48`: `count=2`, `mean=244.6 ms`, `p50=244.6 ms`, `p95=260.2 ms`, `p99=261.6 ms`, `rss_peak_kb=89108`, `hwm_peak_kb=89108`, `memory_samples=19`
