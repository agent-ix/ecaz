# Artifact Manifest

Packet: `review/30068-task28-ivf-prerank-topk`

Measured code SHA: `2ea678c29542d0b5a03a086e9534ade307f4e241`

Timestamp: `2026-04-27T16:45:44-07:00`

Lane: Task 28 IVF scan candidate aggregation, local PG18 only.

Fixture: existing isolated DBPedia-derived n64 surfaces in local PG18 database
`postgres`; `ecaz bench latency`, profile `ec_ivf`, `k=10`, `concurrency=1`,
`iterations=100`, sweep `32,48`.

Storage format: `turboquant`.

Rerank mode: `heap_f32`, `rerank_width=25`.

Surface isolation: one-index-per-table surfaces from packet 30052.

Cache state: warm local development run; no explicit cache drop.

Memory high-water mark: not captured.

## Artifacts

### `latency_10k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30068-task28-ivf-prerank-topk/artifacts/latency_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=96.2 ms p50=95.4 ms p95=104.3 ms p99=108.1 ms`
  - `nprobe=48 count=100 mean=141.9 ms p50=140.4 ms p95=157.4 ms p99=169.6 ms`

### `latency_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30068-task28-ivf-prerank-topk/artifacts/latency_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=240.9 ms p50=240.9 ms p95=254.1 ms p99=261.2 ms`
  - `nprobe=48 count=100 mean=341.8 ms p50=340.3 ms p95=357.3 ms p99=405.6 ms`
