# Artifact Manifest

Packet: `review/30073-task28-ivf-turboquant-lut`

Measured code SHA: `fd7e115da62fe416f36cf9637c81ca24dfeef185`

Timestamp: `2026-04-27T17:35:57-07:00`

Lane: Task 28 IVF TurboQuant no-QJL 4-bit LUT scan, local PG18 only.

Fixture: existing isolated DBPedia-derived n64 10k/25k surfaces in local PG18
database `postgres`; `ecaz bench latency` and `ecaz bench recall`, profile
`ec_ivf`, `k=10`, sweep `32,48`.

Storage format: `turboquant`, using no-QJL 4-bit LUT scoring.

Rerank mode: `heap_f32`, `rerank_width=25`.

Surface isolation: one-index-per-table surfaces from earlier Task 28 packets.

Cache state: warm local development run; no explicit cache drop.

Memory high-water mark: not captured.

## Artifacts

### `latency_10k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30073-task28-ivf-turboquant-lut/artifacts/latency_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 ... p50 60.5 ms ... p95 70.5 ms ... p99 77.5 ms`
  - `48 ... p50 84.5 ms ... p95 122.7 ms ... p99 142.7 ms`

### `latency_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30073-task28-ivf-turboquant-lut/artifacts/latency_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 ... p50 141.6 ms ... p95 162.3 ms ... p99 175.4 ms`
  - `48 ... p50 197.2 ms ... p95 234.2 ms ... p99 265.6 ms`

### `recall_10k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --log-output review/30073-task28-ivf-turboquant-lut/artifacts/recall_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 recall@10 0.9800 ndcg@10 0.9981 mean q-time 60.31 ms`
  - `48 recall@10 1.0000 ndcg@10 1.0000 mean q-time 81.05 ms`

### `recall_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --log-output review/30073-task28-ivf-turboquant-lut/artifacts/recall_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 recall@10 0.9840 ndcg@10 0.9988 mean q-time 137.10 ms`
  - `48 recall@10 0.9990 ndcg@10 1.0000 mean q-time 192.60 ms`
