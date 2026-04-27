# Artifact Manifest

Packet: `review/30070-task28-ivf-borrowed-scan-recall`

Measured code SHA: `86df0a2e74453a27ee203e808d0228a6ca50c5ff`

Timestamp: `2026-04-27T16:57:03-07:00`

Lane: Task 28 IVF borrowed posting scan recall check, local PG18 only.

Fixture: existing isolated DBPedia-derived n64 surfaces in local PG18 database
`postgres`; `ecaz bench recall`, profile `ec_ivf`, `k=10`,
`queries-limit=100`, sweep `32,48`.

Storage format: `turboquant`.

Rerank mode: `heap_f32`, `rerank_width=25`.

Surface isolation: one-index-per-table surfaces from packet 30052.

Cache state: warm local development run; no explicit cache drop.

Memory high-water mark: not captured.

## Artifacts

### `recall_10k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --log-output review/30070-task28-ivf-borrowed-scan-recall/artifacts/recall_10k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 recall@10=0.9800 ndcg@10=0.9981 mean q-time=93.67 ms`
  - `nprobe=48 recall@10=1.0000 ndcg@10=1.0000 mean q-time=134.17 ms`

### `recall_25k_n64w25_nprobe32_48.log`

- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --log-output review/30070-task28-ivf-borrowed-scan-recall/artifacts/recall_25k_n64w25_nprobe32_48.log`
- Key result lines cited:
  - `nprobe=32 recall@10=0.9840 ndcg@10=0.9988 mean q-time=233.10 ms`
  - `nprobe=48 recall@10=0.9990 ndcg@10=1.0000 mean q-time=330.18 ms`
