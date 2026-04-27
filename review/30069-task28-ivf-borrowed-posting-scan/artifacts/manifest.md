# Artifact Manifest

Packet: `review/30069-task28-ivf-borrowed-posting-scan`

Measured code SHA: `30d9ffc8920c7de3379c97a9f290866095940471`

Timestamp: `2026-04-27T16:52:09-07:00`

Lane: Task 28 IVF scan posting tuple decode, local PG18 only.

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

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30069-task28-ivf-borrowed-posting-scan/artifacts/latency_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=93.9 ms p50=93.4 ms p95=102.6 ms p99=103.6 ms`
  - `nprobe=48 count=100 mean=139.1 ms p50=136.1 ms p95=166.8 ms p99=174.6 ms`

### `latency_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30069-task28-ivf-borrowed-posting-scan/artifacts/latency_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=233.2 ms p50=234.2 ms p95=248.8 ms p99=254.6 ms`
  - `nprobe=48 count=100 mean=330.9 ms p50=329.9 ms p95=346.6 ms p99=373.9 ms`
