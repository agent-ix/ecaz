# Artifact Manifest: 30143 Task 28 IVF A10 TurboQuant Memory

## `latency_turboquant_10k_n64_w750_p48_hwm.log`

- head SHA: `a07f52dd`
- packet/topic: `30143-task28-ivf-a10-turboquant-memory`
- lane / fixture / storage format / rerank mode: A10 10k TurboQuant, `nlists=64`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30143-task28-ivf-a10-turboquant-memory/artifacts/latency_turboquant_10k_n64_w750_p48_hwm.log`
- timestamp: 2026-04-28 local
- isolated/shared surface: isolated existing 10k TurboQuant surface
- key result lines:
  - `p50=130.6 ms`, `p95=231.6 ms`, `p99=267.9 ms`
  - `rss_peak_kb=109600`, `hwm_peak_kb=109600`

## `latency_turboquant_25k_n64_w750_p48_hwm.log`

- head SHA: `a07f52dd`
- packet/topic: `30143-task28-ivf-a10-turboquant-memory`
- lane / fixture / storage format / rerank mode: A10 25k TurboQuant, `nlists=64`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30143-task28-ivf-a10-turboquant-memory/artifacts/latency_turboquant_25k_n64_w750_p48_hwm.log`
- timestamp: 2026-04-28 local
- isolated/shared surface: isolated existing 25k TurboQuant surface
- key result lines:
  - `p50=284.5 ms`, `p95=402.4 ms`, `p99=441.5 ms`
  - `rss_peak_kb=155540`, `hwm_peak_kb=155540`
