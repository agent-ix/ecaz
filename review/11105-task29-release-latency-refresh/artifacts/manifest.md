# Artifact Manifest

Packet: `11105-task29-release-latency-refresh`
Timestamp: `2026-04-30T16:38:18-07:00`

Primary code head for new CLI and pgvectorscale comparison:
`8064bf51b339c3b26f69354e325c62d99a57d84a`

DiskANN release recall/latency artifacts were collected before the CLI-only
comparison commit, with the same `ec_diskann` extension code and the release
PG18 extension installed from this branch. The release install command is
captured again in `install-tqvector-pg18-release-escalated.log`.

## Environment

- PostgreSQL: local pgrx PG18 scratch server, socket directory
  `/home/peter/.pgrx`, port `28818`.
- Database: `task29_diskann_baseline`.
- Corpus prefix: `task29c_phase_profile`.
- Corpus shape: real-10k corpus, 200 queries, dim 1536.
- Storage format: `ec_diskann` `pq_fastscan` tuple format with binary sidecar.
- Rerank mode: `ec_diskann` exact heap-source rerank after graph traversal.
- Measurement surface: isolated one-index-per-table for DiskANN latency after
  dropping HNSW reference indexes; pgvectorscale uses a sidecar table.
- Cache state: warm local scratch server after prior corpus/index setup;
  `EXPLAIN` artifact records buffer hits for pgvectorscale L=200.

## Artifacts

### `install-tqvector-pg18-release.log` and `install-tqvector-pg18-release-escalated.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: release PG18 extension install.
- Commands:
  - sandbox attempt:
    `script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/11105-task29-release-latency-refresh/artifacts/install-tqvector-pg18-release.log`
  - escalated retry:
    `script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/11105-task29-release-latency-refresh/artifacts/install-tqvector-pg18-release-escalated.log`
- Key result: sandbox attempt proved the release build command but failed to
  write into the PG18 install tree; escalated retry copied `ecaz.so`, wrote
  `ecaz--0.1.1.sql`, and finished installing `ecaz`.

### `drop-task29c-hnsw-reference-indexes-before-release-sweep.log`

- Head SHA: pre-CLI code head on this branch.
- Lane / fixture: isolate existing `task29c_phase_profile` DiskANN index by
  removing HNSW reference indexes.
- Command:
  `target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11105-task29-release-latency-refresh/artifacts/drop-task29c-hnsw-reference-indexes-before-release-sweep.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_m32_idx; DROP INDEX IF EXISTS task29c_hnsw_reference_idx;"`
- Key result: both indexes absent or dropped.

### `recall-diskann-release-table.log`

- Head SHA: pre-CLI code head on this branch.
- Lane / fixture: `ec_diskann`, real-10k, `k=10`, list-size sweep
  `64,128,200,400,800`, forced index path, shared table with only DiskANN index.
- Command:
  `target/release/ecaz bench recall --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-dir review/11105-task29-release-latency-refresh/artifacts --log-file review/11105-task29-release-latency-refresh/artifacts/recall-diskann-release-cli.log --log-output review/11105-task29-release-latency-refresh/artifacts/recall-diskann-release-table.log`
- Key result lines:
  - L=64 recall@10 `0.9965`, NDCG `0.9999`, mean q-time `8.34 ms`
  - L=128 recall@10 `0.9965`, NDCG `0.9999`, mean q-time `8.43 ms`
  - L=200 recall@10 `0.9970`, NDCG `0.9999`, mean q-time `8.57 ms`
  - L=400 recall@10 `0.9970`, NDCG `0.9999`, mean q-time `9.03 ms`
  - L=800 recall@10 `0.9975`, NDCG `0.9999`, mean q-time `10.36 ms`

### `latency-diskann-release-table.log`

- Head SHA: pre-CLI code head on this branch.
- Lane / fixture: `ec_diskann`, real-10k, `k=10`, 200 iterations,
  concurrency 1, list-size sweep `64,128,200,400,800`, backend memory sampled.
- Command:
  `target/release/ecaz bench latency --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-file review/11105-task29-release-latency-refresh/artifacts/latency-diskann-release-cli.log --log-output review/11105-task29-release-latency-refresh/artifacts/latency-diskann-release-table.log`
- Key result lines:
  - L=64 mean/p50/p95/p99 `8.05/7.98/8.74/9.48 ms`, HWM `67104 KiB`
  - L=128 mean/p50/p95/p99 `8.20/8.14/8.97/9.53 ms`, HWM `67240 KiB`
  - L=200 mean/p50/p95/p99 `8.74/8.64/9.44/11.5 ms`, HWM `67452 KiB`
  - L=400 mean/p50/p95/p99 `9.02/8.93/9.82/10.4 ms`, HWM `67824 KiB`
  - L=800 mean/p50/p95/p99 `9.57/9.50/10.6/11.3 ms`, HWM `68688 KiB`

### `truth-v1-rows10000-queries200-dim1536-k10-4473cd157aa35fa6.json`

- Head SHA: pre-CLI code head on this branch.
- Lane / fixture: packet-local exact truth cache for DiskANN release recall.
- Command: produced by `bench recall` above with
  `--truth-cache-dir review/11105-task29-release-latency-refresh/artifacts`.

### `current-cargo-pgrx-version.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: pgvectorscale setup prerequisite check.
- Command:
  `script -q -e -c "cargo-pgrx pgrx --version" review/11105-task29-release-latency-refresh/artifacts/current-cargo-pgrx-version.log`
- Key result: global cargo-pgrx is `0.17.0`.

### `install-cargo-pgrx-0.16.1.log` and `install-cargo-pgrx-0.16.1-escalated.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: isolated pgvectorscale build tool install.
- Commands:
  - sandbox attempt:
    `script -q -e -c "cargo install --locked cargo-pgrx --version 0.16.1 --root /tmp/pgvectorscale-cargo-pgrx-0.16.1" review/11105-task29-release-latency-refresh/artifacts/install-cargo-pgrx-0.16.1.log`
  - escalated retry:
    `script -q -e -c "cargo install --locked cargo-pgrx --version 0.16.1 --root /tmp/pgvectorscale-cargo-pgrx-0.16.1" review/11105-task29-release-latency-refresh/artifacts/install-cargo-pgrx-0.16.1-escalated.log`
- Key result: first attempt failed on network DNS; escalated retry installed
  `/tmp/pgvectorscale-cargo-pgrx-0.16.1/bin/cargo-pgrx`.

### `install-pgvectorscale-release.log`

- Head SHA: local pgvectorscale checkout
  `4c04103b6e21d4ee920e41d5ad0a10178c6af1b3`.
- Lane / fixture: pgvectorscale `0.9.0` release install into PG18.
- Command:
  `script -q -e -c "/tmp/pgvectorscale-cargo-pgrx-0.16.1/bin/cargo-pgrx pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config" /home/peter/dev/tqvector/review/11105-task29-release-latency-refresh/artifacts/install-pgvectorscale-release.log`
- Key result: built release `vectorscale v0.9.0`; copied
  `vectorscale-0.9.0.so`; wrote `vectorscale--0.9.0.sql`.

### `rebuild-pgvector-pg18-clean.log` and `rebuild-pgvector-pg18-install.log`

- Head SHA: local pgvector checkout
  `17916cad00ee580b05372768d6ff84b61442166a`.
- Lane / fixture: fix stale PG17-built pgvector library in PG18 install tree.
- Commands:
  - `script -q -e -c "make clean PG_CONFIG=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config" /home/peter/dev/tqvector/review/11105-task29-release-latency-refresh/artifacts/rebuild-pgvector-pg18-clean.log`
  - `script -q -e -c "make install PG_CONFIG=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config" /home/peter/dev/tqvector/review/11105-task29-release-latency-refresh/artifacts/rebuild-pgvector-pg18-install.log`
- Key result: rebuilt `vector.so` with PG18 include/lib paths and installed
  `vector--0.8.2.sql`.

### `check-available-vector-extensions*.log`

- Head SHA: mixed setup checks around pgvectorscale install.
- Lane / fixture: `pg_available_extensions` checks via `ecaz dev sql`.
- Key result:
  - Before external installs: no vector/vectorscale rows.
  - After pgvector rebuild and pgvectorscale install: `vector 0.8.2` and
    `vectorscale 0.9.0` available.

### `compare-vectorscale-release-cli-rerun.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: pgvectorscale head-to-head on real-10k, sidecar table,
  matched sweep `64,128,200,400,800`.
- Storage format / rerank mode: pgvectorscale `diskann` AM on pgvector
  `vector(1536)` with `vector_ip_ops`, `storage_layout=memory_optimized`,
  `diskann.query_rescore` defaulting to the sweep value.
- Command:
  `target/release/ecaz compare vectorscale --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --vectorscale-num-neighbors 32 --vectorscale-build-search-list-size 100 --vectorscale-max-alpha 1.2 --rebuild --log-file review/11105-task29-release-latency-refresh/artifacts/compare-vectorscale-release-cli-rerun.log`
- Key result lines:
  - pgvectorscale sidecar populated `10000` rows.
  - pgvectorscale build time `5.82s`.
  - pgvectorscale index size `5136384` bytes.
  - L=64: `ec_diskann` recall/mean `0.9965` / `9.19 ms`;
    pgvectorscale `0.9960` / `3.56 ms`
  - L=128: `ec_diskann` `0.9965` / `8.06 ms`;
    pgvectorscale `0.9990` / `5.84 ms`
  - L=200: `ec_diskann` `0.9970` / `10.4 ms`;
    pgvectorscale `1.0000` / `8.85 ms`
  - L=400: `ec_diskann` `0.9970` / `9.86 ms`;
    pgvectorscale `1.0000` / `16.2 ms`
  - L=800: `ec_diskann` `0.9975` / `10.1 ms`;
    pgvectorscale `1.0000` / `31.2 ms`

### `compare-vectorscale-release-cli.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: failed first pgvectorscale comparison attempt.
- Key result: failed on `CREATE EXTENSION vector` because the PG18 install
  tree had a stale PG17-built `vector.so`; corrected by the pgvector rebuild
  artifacts above.

### `explain-vectorscale-l200-release.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: pgvectorscale L=200 access-path confirmation.
- Command:
  `target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11105-task29-release-latency-refresh/artifacts/explain-vectorscale-l200-release.log --sql "CREATE EXTENSION IF NOT EXISTS vector; CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE; SET enable_seqscan = off; SET diskann.query_search_list_size = 200; SET diskann.query_rescore = 200; EXPLAIN (ANALYZE, BUFFERS) SELECT id FROM task29c_phase_profile_corpus_vectorscale ORDER BY embedding <#> (SELECT source::vector(1536) FROM task29c_phase_profile_queries ORDER BY id LIMIT 1) LIMIT 10;"`
- Key result: `Index Scan using task29c_phase_profile_corpus_vectorscale_diskann_idx`;
  execution time `12.120 ms`; buffers `shared hit=5289 read=1`.

### `storage-release-indexes.log`

- Head SHA: `8064bf51b339c3b26f69354e325c62d99a57d84a`
- Lane / fixture: release index-size comparison via `ecaz dev sql`.
- Command:
  `target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11105-task29-release-latency-refresh/artifacts/storage-release-indexes.log --sql "SELECT c.relname, am.amname, pg_size_pretty(pg_relation_size(c.oid)) AS size, pg_relation_size(c.oid) AS bytes FROM pg_class c JOIN pg_index i ON i.indexrelid = c.oid JOIN pg_am am ON am.oid = c.relam WHERE c.relname IN ('task29c_phase_profile_idx', 'task29c_phase_profile_corpus_vectorscale_diskann_idx') ORDER BY c.relname;"`
- Key result:
  - pgvectorscale `diskann`: `5016 kB`, `5136384` bytes
  - `ec_diskann`: `4824 kB`, `4939776` bytes
