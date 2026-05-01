# Artifact Manifest

Packet: `review/11109-task29d-final-readiness`
Head SHA before packet: `bc44adc5ade0fe366396b88897cd58dd08b74510`
Timestamp: `2026-04-30T22:14:29-07:00`
Lane: Task 29d final readiness
Fixture: local real-10k corpus, prefix `task29c_phase_profile`
PostgreSQL: local pgrx PG18 scratch server, release-installed `ecaz`
Connection targeting: `ecaz` CLI flags with socket directory and port

## Release Install

Preparation command:

```sh
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18
```

## Initial Index Check

Artifact: `check-final-indexes.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11109-task29d-final-readiness/artifacts/check-final-indexes.log --sql "SELECT c.relname, am.amname, pg_size_pretty(pg_relation_size(c.oid)) AS relation_size, pg_relation_size(c.oid) AS bytes FROM pg_class c JOIN pg_am am ON am.oid = c.relam WHERE c.relname IN ('task29c_phase_profile_idx', 'task29c_phase_profile_m32_idx', 'task29c_hnsw_reference_idx', 'task29c_phase_profile_corpus_vectorscale_diskann_idx') ORDER BY c.relname;"
```

Key result:

- Present before final rebuilds: `task29c_phase_profile_idx`, `task29c_phase_profile_corpus_vectorscale_diskann_idx`.

## pgvectorscale

Artifact: `compare-vectorscale-final.log`

Command:

```sh
target/release/ecaz compare vectorscale --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --vectorscale-num-neighbors 32 --vectorscale-build-search-list-size 100 --vectorscale-max-alpha 1.2 --rebuild --log-file review/11109-task29d-final-readiness/artifacts/compare-vectorscale-final.log
```

Key result lines:

- build: `built task29c_phase_profile_corpus_vectorscale_diskann_idx in 5.72s`
- size: `pg_relation_size=5136384 bytes`
- L=64: `recall@k=0.9955 mean=3.48 ms p99=4.49 ms`
- L=128: `recall@k=0.9990 mean=5.81 ms p99=6.74 ms`
- L=200: `recall@k=1.0000 mean=8.50 ms p99=10.2 ms`
- L=400: `recall@k=1.0000 mean=17.3 ms p99=22.2 ms`
- L=800: `recall@k=1.0000 mean=30.1 ms p99=33.7 ms`

Note: ec_diskann rows in this compare artifact are not cited because the shared
table still had an HNSW index, and the direct isolated ec_diskann artifacts
below are the source of truth.

## ec_diskann

Isolation artifact: `drop-hnsw-before-diskann-final.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11109-task29d-final-readiness/artifacts/drop-hnsw-before-diskann-final.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_m32_idx; DROP INDEX IF EXISTS task29c_hnsw_reference_idx;"
```

Recall artifacts:

- `recall-diskann-final-isolated-table.log`
- `recall-diskann-final-isolated-cli.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 bench recall --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11109-task29d-final-readiness/artifacts/truth-v1-rows10000-queries200-dim1536-k10-4473cd157aa35fa6.json --log-file review/11109-task29d-final-readiness/artifacts/recall-diskann-final-isolated-cli.log --log-output review/11109-task29d-final-readiness/artifacts/recall-diskann-final-isolated-table.log
```

Key recall lines:

- L=64: `recall@k=0.9965 ndcg@k=0.9999 mean q-time=8.07 ms`
- L=128: `recall@k=0.9965 ndcg@k=0.9999 mean q-time=7.99 ms`
- L=200: `recall@k=0.9970 ndcg@k=0.9999 mean q-time=8.12 ms`
- L=400: `recall@k=0.9970 ndcg@k=0.9999 mean q-time=8.73 ms`
- L=800: `recall@k=0.9975 ndcg@k=0.9999 mean q-time=9.55 ms`

Latency artifacts:

- `latency-diskann-final-isolated-after-restart-table.log`
- `latency-diskann-final-isolated-after-restart-cli.log`

Command:

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 bench latency --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 500 --concurrency 1 --force-index --sample-backend-memory --log-file review/11109-task29d-final-readiness/artifacts/latency-diskann-final-isolated-after-restart-cli.log --log-output review/11109-task29d-final-readiness/artifacts/latency-diskann-final-isolated-after-restart-table.log
```

Key latency lines:

- L=64: `mean=7.80 ms p50=7.63 ms p95=8.99 ms p99=10.3 ms hwm_peak_kb=61180`
- L=128: `mean=7.79 ms p50=7.67 ms p95=8.44 ms p99=10.2 ms hwm_peak_kb=61500`
- L=200: `mean=7.98 ms p50=7.87 ms p95=8.83 ms p99=10.3 ms hwm_peak_kb=61980`
- L=400: `mean=8.49 ms p50=8.40 ms p95=9.39 ms p99=10.8 ms hwm_peak_kb=62840`
- L=800: `mean=9.34 ms p50=9.23 ms p95=10.4 ms p99=12.9 ms hwm_peak_kb=63404`

Ignored audit artifacts:

- `recall-diskann-final-table.log`
- `recall-diskann-final-cli.log`
- `latency-diskann-final-isolated-table.log`
- `latency-diskann-final-isolated-cli.log`

The first recall pair was run while HNSW was still present; the first isolated
latency pair was run before restarting PG18 and showed inflated backend HWM.

## ec_hnsw

Isolation artifact: `drop-diskann-before-hnsw-final.log`

Build artifact: `load-final-hnsw-reference-isolated.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11109-task29d-final-readiness/artifacts/load-final-hnsw-reference-isolated.log corpus load --prefix task29c_phase_profile --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_hnsw --m 32 --ef-construction 100 --allow-manifest-mismatch
```

Key build line:

- `built task29c_phase_profile_m32_idx in 5.77s`

Recall artifacts:

- `recall-hnsw-final-isolated-table.log`
- `recall-hnsw-final-isolated-cli.log`

Latency artifacts:

- `latency-hnsw-final-isolated-table.log`
- `latency-hnsw-final-isolated-cli.log`

Key result lines:

- ef=64: `recall@k=0.9695 mean=2.91 ms p99=4.78 ms`
- ef=128: `recall@k=0.9710 mean=4.75 ms p99=6.83 ms`
- ef=200: `recall@k=0.9710 mean=6.75 ms p99=8.58 ms`
- ef=400: `recall@k=0.9715 mean=13.0 ms p99=18.0 ms`
- ef=800: `recall@k=0.9715 mean=25.5 ms p99=41.1 ms`

## Final Restore And Storage

Artifact: `rebuild-diskann-after-hnsw-final.log`

Key result lines:

- pass 0: `elapsed_ms=4450`
- pass 1: `elapsed_ms=8318`
- complete: `build_persist_ms=12994 core_graph_ms=12770 total_ms=14590`

Artifact: `storage-final-indexes.log`

Key result lines:

- `task29c_phase_profile_corpus_vectorscale_diskann_idx diskann 5016 kB 5136384`
- `task29c_phase_profile_idx ec_diskann 4824 kB 4939776`
- `task29c_phase_profile_m32_idx ec_hnsw 14 MB 15130624`
