# Artifact Manifest

Packet: `review/30067-task28-ivf-prepared-quantizer-scan`

Packet code SHA: `1fb3167fc9905d30094296428b526459f06a4aeb`

Timestamp: `2026-04-27T16:38:51-07:00`

Lane: Task 28 IVF scan hot-path A/B, local PG18 only.

Fixture: existing isolated `task28_ivf_postopt10k_n64w25` 10k x 1536 surface
in local PG18 database `postgres`; `ecaz bench latency`, profile `ec_ivf`,
`k=10`, `concurrency=1`, `iterations=100`, sweep `32,48`.

Storage format: `turboquant`.

Rerank mode: `heap_f32`, `rerank_width=25`.

Surface isolation: one-index-per-table surface from packet 30052.

Cache state: warm local development run; no explicit cache drop.

Memory high-water mark: not captured.

## Artifacts

### `latency_10k_n64w25_nprobe32_48.log`

- Code state: uncommitted prepared-quantizer trial on top of the Task 28 branch,
  with `IvfPreparedQuery` carrying `Arc<ProdQuantizer>`.
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30067-task28-ivf-prepared-quantizer-scan/artifacts/latency_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=116.3 ms p50=111.8 ms p95=147.2 ms p99=159.0 ms`
  - `nprobe=48 count=100 mean=156.8 ms p50=156.1 ms p95=166.5 ms p99=172.8 ms`

### `latency_10k_n64w25_nprobe32_48_r2.log`

- Code state: same uncommitted prepared-quantizer trial.
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30067-task28-ivf-prepared-quantizer-scan/artifacts/latency_10k_n64w25_nprobe32_48_r2.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=108.8 ms p50=108.4 ms p95=115.6 ms p99=120.6 ms`
  - `nprobe=48 count=100 mean=156.4 ms p50=156.0 ms p95=165.6 ms p99=167.9 ms`

### `latency_10k_n64w25_nprobe32_48_ab_baseline.log`

- Code state: prepared-quantizer trial backed out; equivalent to committed
  code SHA `1fb3167fc9905d30094296428b526459f06a4aeb`.
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --log-output review/30067-task28-ivf-prepared-quantizer-scan/artifacts/latency_10k_n64w25_nprobe32_48_ab_baseline.log`
- Key result lines cited:
  - `nprobe=32 count=100 mean=99.4 ms p50=97.7 ms p95=119.6 ms p99=125.6 ms`
  - `nprobe=48 count=100 mean=140.2 ms p50=139.1 ms p95=150.1 ms p99=175.6 ms`
