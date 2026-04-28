# Artifact Manifest

Packet: `review/30072-task28-ivf-frontier-prune`

Base code SHA: `13292ecb6ad4cb84fe54ece0212a6794b9b5914a`

Measured code: base SHA plus uncommitted trial diff
`artifacts/frontier_prune_trial.diff`. The trial was reverted before commit.

Timestamp: `2026-04-27T17:28:57-07:00`

Lane: Task 28 IVF post-score frontier prune trial, local PG18 only.

Fixture: existing isolated DBPedia-derived n64 10k/25k surfaces in local PG18
database `postgres`; `ecaz bench latency` and `ecaz bench recall`, profile
`ec_ivf`, `k=10`, sweep `32,48`.

Storage format: `turboquant`.

Rerank mode: `heap_f32`, `rerank_width=25`.

Surface isolation: one-index-per-table surfaces from earlier Task 28 packets.

Cache state: warm local development run; no explicit cache drop.

Memory high-water mark: not captured.

## Artifacts

### `frontier_prune_trial.diff`

- Source diff for the trial that was measured and then reverted.

### `latency_10k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30072-task28-ivf-frontier-prune/artifacts/latency_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 ... p50 95.1 ms ... p95 110.2 ms ... p99 119.6 ms`
  - `48 ... p50 135.9 ms ... p95 158.1 ms ... p99 172.0 ms`

### `latency_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30072-task28-ivf-frontier-prune/artifacts/latency_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 ... p50 243.5 ms ... p95 267.4 ms ... p99 277.7 ms`
  - `48 ... p50 341.7 ms ... p95 399.9 ms ... p99 413.2 ms`

### `recall_10k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --log-output review/30072-task28-ivf-frontier-prune/artifacts/recall_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 recall@10 0.9800 ndcg@10 0.9981 mean q-time 94.93 ms`
  - `48 recall@10 1.0000 ndcg@10 1.0000 mean q-time 132.86 ms`

### `recall_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --log-output review/30072-task28-ivf-frontier-prune/artifacts/recall_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `32 recall@10 0.9840 ndcg@10 0.9988 mean q-time 239.08 ms`
  - `48 recall@10 0.9990 ndcg@10 1.0000 mean q-time 326.67 ms`
