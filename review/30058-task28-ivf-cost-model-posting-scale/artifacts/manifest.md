# Artifact Manifest

Packet: `review/30058-task28-ivf-cost-model-posting-scale`

Head SHA: `077aae15e01113fc41aced09dbb6624bec84ccb1`

Timestamp: `2026-04-27T15:25:36-07:00`

Lane: Task 28 IVF cost-model repair, local PG18 only.

Fixture: existing isolated one-index-per-table surface
`task28_ivf_postopt10k_n128w25`, 10k corpus rows, 100 query rows,
1536-dimensional DBPedia-derived real source vectors.

Storage / rerank: `ec_ivf`, `storage_format=turboquant`, `nlists=128`,
`rerank_width=25`, `rerank=heap_f32`, session `ec_ivf.nprobe=8` for measured
queries.

## Environment Metadata

- File: `metadata.log`
- Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30058-task28-ivf-cost-model-posting-scale/artifacts/metadata.sql --raw --log-output review/30058-task28-ivf-cost-model-posting-scale/artifacts/metadata.log`
- Key lines:
  - `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
  - `shared_buffers = 128MB`
  - `random_page_cost = 4`
  - `seq_page_cost = 1`
  - `cpu_operator_cost = 0.0025`
  - `index_bytes = 10166272`
  - `corpus_rows = 10000`
  - `query_rows = 100`

## Prepared EXPLAIN

- File: `explain_n128_nprobe8_prepared.log`
- SQL: `explain_n128_nprobe8_prepared.sql`
- Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30058-task28-ivf-cost-model-posting-scale/artifacts/explain_n128_nprobe8_prepared.sql --raw --log-output review/30058-task28-ivf-cost-model-posting-scale/artifacts/explain_n128_nprobe8_prepared.log`
- Surface isolation: isolated one-index-per-table.
- Key lines:
  - `Limit  (cost=43.00..43.55 rows=10 width=12) (actual time=60.761..64.569 rows=10.00 loops=1)`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx on task28_ivf_postopt10k_n128w25_corpus  (cost=43.00..594.25 rows=10000 width=12)`
  - `Buffers: shared hit=636`
  - `Execution Time: 64.663 ms`

## Recall Smoke

- File: `recall_10k_n128w25_nprobe8_normal.log`
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n128w25 --profile ec_ivf --k 10 --queries-limit 20 --sweep 8`
- Surface isolation: isolated one-index-per-table.
- Note: `bench recall` does not yet expose `--log-output`; the artifact file
  preserves the direct command output captured during the run.
- Key lines:
  - `ground truth in 872.70ms`
  - `8 | 0.7000 | 0.9723 | 40.98 ms`

## Latency Smoke

- File: `latency_10k_n128w25_nprobe8_normal.log`
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n128w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 20 --sweep 8`
- Surface isolation: isolated one-index-per-table.
- Note: `bench latency` does not yet expose `--log-output`; the artifact file
  preserves the direct command output captured during the run.
- Key lines:
  - `8 | 20 | 34.8 ms | 8.03 ms | 28.1 ms | 33.5 ms | 39.3 ms | 62.3 ms | 68.1 ms`
