# Artifact Manifest

Head SHA: `6491aeb60a6905ff546f117ce5d6d14d032059b4`

Packet: `review/11096-task29a-diskann-binary-sidecar-prefilter`

Lane: Task 29a DiskANN binary-sidecar prefilter, local PG18.

Fixture: real-10k, 1536-d, 10k corpus rows, 200 query rows.

Storage format: `ec_diskann` `pq_fastscan` tuple format with persisted
binary sidecar present; scan prefilter default `auto`.

Rerank mode: heap-f32 exact rerank, reloption default `rerank_budget=64`
unless the diagnostic command explicitly reports otherwise.

Table model: isolated one-index-per-table prefixes:
`task29a_sidecar_real10k` / `task29_diskann_real10k` for DiskANN and
`task29_hnsw_real10k` for HNSW reference.

Cache state: local PG18 was warm by the end of the packet run; see
`cache-state-check.log`.

## Artifacts

### `guc-reloptions-check.log`

Command:

`cargo run -p ecaz-cli -- --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/guc-reloptions-check.log --sql "LOAD 'ecaz'; SHOW ec_diskann.list_size; SHOW ec_diskann.prefilter_kind; SELECT relname, reloptions FROM pg_class WHERE relname IN ('task29_diskann_real10k_idx','task29_diskann_real10k_corpus_embedding_idx') ORDER BY relname;"`

Timestamp: 2026-04-30 local.

Key lines:

- `ec_diskann.list_size = -1`
- `ec_diskann.prefilter_kind = auto`
- `task29_diskann_real10k_idx {graph_degree=32,build_list_size=100,alpha=1.2}`

### `load-task29a-sidecar-real10k.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/load-task29a-sidecar-real10k.log corpus load --prefix task29a_sidecar_real10k --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --allow-manifest-mismatch`

Timestamp: 2026-04-30 local.

Key lines:

- `copied corpus table task29a_sidecar_real10k_corpus in 4.27s`
- `encoded corpus table task29a_sidecar_real10k_corpus in 4.55s`
- `built task29a_sidecar_real10k_idx in 492.13s`
- `completed prefix task29a_sidecar_real10k in 503.10s`

### `recall-task29a-sidecar-fresh-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/recall-task29a-sidecar-fresh-cli.log bench recall --prefix task29a_sidecar_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/recall-task29a-sidecar-fresh-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: recall@10 `0.9965`, NDCG `0.9999`, mean `51.13 ms`
- L=128: recall@10 `0.9965`, NDCG `0.9999`, mean `58.55 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `70.23 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `114.45 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `258.79 ms`

### `sql-vs-memory-sidecar-auto.txt`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/sql-vs-memory-sidecar-auto-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --scan-list-size 200 --recall-k 10 --compare-queries 5 --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/sql-vs-memory-sidecar-auto.txt`

Timestamp: 2026-04-30 local.

Key lines:

- In-memory graph recall@10: `1.0000`
- Query `10001`: `exact/sql = 10/10`, SQL IDs include
  `8885,9785,9957,9826,9717,9926,9944,9855,9915,7782`

### `frontier-binary-sidecar-q10001.txt`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/frontier-binary-sidecar-q10001-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --scan-list-size 200 --recall-k 10 --frontier-query-id 10001 --frontier-prefilter binary-sidecar --frontier-rerank-budget 64 --frontier-top 80 --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/frontier-binary-sidecar-q10001.txt`

Timestamp: 2026-04-30 local.

Key lines:

- `frontier_prefilter = binary_sidecar`
- `reranked_ids = 8885,9785,9957,9826,9717,9926,9944,9855,9915,7782`
- ID `9717`: exact rank `5`, frontier rank `25`, in rerank budget `true`
- ID `7782`: exact rank `10`, frontier rank `47`, in rerank budget `true`

### `recall-sidecar-auto-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/recall-sidecar-auto-cli.log bench recall --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/recall-sidecar-auto-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: recall@10 `0.9955`, NDCG `0.9997`, mean `52.87 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `67.65 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `247.34 ms`

### `latency-sidecar-auto-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/latency-sidecar-auto-cli.log bench latency --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/latency-sidecar-auto-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: mean `48.5 ms`, p50 `48.5 ms`, p95 `51.0 ms`, p99 `52.4 ms`, HWM `70148 KiB`
- L=200: mean `66.5 ms`, p50 `66.6 ms`, p95 `71.7 ms`, p99 `73.5 ms`, HWM `70468 KiB`
- L=800: mean `250.1 ms`, p50 `249.9 ms`, p95 `278.6 ms`, p99 `305.1 ms`, HWM `70948 KiB`

### `storage-task29a-sidecar-fresh-cli.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/storage-task29a-sidecar-fresh-cli.log bench storage --prefix task29a_sidecar_real10k`

Timestamp: 2026-04-30 local.

Key lines:

- Fresh DiskANN index `task29a_sidecar_real10k_idx`: `4.7 MiB`, `494.0 B` per row
- Total table + indexes: `164.5 MiB`, `17253.2 B` per row

### `recall-ec-hnsw-reference-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/recall-ec-hnsw-reference-cli.log bench recall --prefix task29_hnsw_real10k --profile ec_hnsw --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/recall-ec-hnsw-reference-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- ef=64: recall@10 `0.9305`, NDCG `0.9814`, mean `18.44 ms`
- ef=200: recall@10 `0.9700`, NDCG `0.9993`, mean `35.25 ms`
- ef=800: recall@10 `0.9720`, NDCG `0.9995`, mean `113.08 ms`

### `latency-ec-hnsw-reference-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/latency-ec-hnsw-reference-cli.log bench latency --prefix task29_hnsw_real10k --profile ec_hnsw --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/latency-ec-hnsw-reference-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- ef=64: mean `17.1 ms`, p50 `16.2 ms`, p95 `21.8 ms`, p99 `25.8 ms`, HWM `47748 KiB`
- ef=200: mean `34.1 ms`, p50 `33.1 ms`, p95 `39.4 ms`, p99 `49.1 ms`, HWM `49028 KiB`
- ef=800: mean `111.2 ms`, p50 `110.0 ms`, p95 `131.3 ms`, p99 `140.0 ms`, HWM `50532 KiB`

### `storage-ec-hnsw-reference-cli.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/storage-ec-hnsw-reference-cli.log bench storage --prefix task29_hnsw_real10k`

Timestamp: 2026-04-30 local.

Key lines:

- HNSW index `task29_hnsw_real10k_m16_idx`: `13.0 MiB`, `1366.4 B` per row
- Total table + indexes: `172.9 MiB`, `18125.6 B` per row

### `cache-state-check.log`

Command:

`cargo run -p ecaz-cli -- --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/cache-state-check.log --sql "LOAD 'ecaz'; SELECT datname, blks_read, blks_hit, round(100.0 * blks_hit / nullif(blks_hit + blks_read, 0), 2) AS hit_pct FROM pg_stat_database WHERE datname = current_database(); SHOW shared_buffers; SHOW effective_cache_size;"`

Timestamp: 2026-04-30 local.

Key lines:

- `task29_diskann_baseline 265945 59965143 99.56`
- `shared_buffers = 128MB`
- `effective_cache_size = 4GB`
