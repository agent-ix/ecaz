# Artifact Manifest

Packet: `review/11108-task29d-build-distance-simd`
Measurement/code SHA: `0cd4baf9`
Packet/plan SHA at write time: `6704229729b380715f838001c63c204969641b79`
Timestamp: `2026-04-30T22:06:10-07:00`
Lane: Task 29d build-performance attack
Fixture: local real-10k corpus, prefix `task29c_phase_profile`
Storage format: ec_diskann index over `task29c_phase_profile_corpus`
Index surface: isolated one-index-per-table
Rerank mode: exact heap rerank, default `rerank_budget=64`
Cache state: local PG18 scratch server; latency table cites the after-restart run
Connection targeting: `ecaz` CLI flags with PG18 socket directory and port

## Helper Benchmark

Artifact: `simd-bench-before.log`

Command:

```sh
cargo run --release --bin simd_bench -- --iterations 1000000 --log-output tmp/task29d-simd-bench-before.log
```

Key result lines:

- `backend=avx2+fma`
- `f32_inner_product/d1536: total=1.23880325s ns_per_iter=1238.8`
- `score_ip_encoded/d1536_b4: total=1.288404007s ns_per_iter=1288.4`

## Release Build Measurement

Artifact: `create-index-task29d-build-distance-simd-release.log`

Preparation:

```sh
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18
```

Command:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11108-task29d-build-distance-simd/artifacts/create-index-task29d-build-distance-simd-release.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"
```

Key result lines:

- pass 0: `elapsed_ms=4452 greedy_search_ms=1 greedy_distance_calls=9651886`
- pass 1: `elapsed_ms=8185 greedy_search_ms=165 robust_prune_ms=0 backlink_ms=38 greedy_distance_calls=12864088 robust_prune_distance_calls=17836674`
- complete: `build_persist_ms=12855 core_medoid_ms=209 core_graph_ms=12639 total_ms=14493`

Comparator:

- Packet `11104` active-mask release baseline: `total_ms=70678`, `build_persist_ms=69000`, `core_graph_ms=67571`, pass 0 `20737 ms`, pass 1 `46832 ms`.

## Size

Artifact: `size-build-distance-simd.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11108-task29d-build-distance-simd/artifacts/size-build-distance-simd.log --sql "SELECT relname, pg_size_pretty(pg_relation_size(oid)) AS relation_size, pg_relation_size(oid) AS bytes FROM pg_class WHERE relname IN ('task29c_phase_profile_idx') ORDER BY relname;"
```

Key result line:

- `task29c_phase_profile_idx 4824 kB 4939776`

## Recall

Artifacts:

- `recall-build-distance-simd-table.log`
- `recall-build-distance-simd-cli.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 bench recall --prefix task29c_phase_profile --profile ec_diskann --sweep 64,200,800 --truth-cache-file review/11107-task29d-l64-scan-profile/artifacts/truth-v1-rows10000-queries200-dim1536-k10-4473cd157aa35fa6.json --log-file review/11108-task29d-build-distance-simd/artifacts/recall-build-distance-simd-cli.log --log-output review/11108-task29d-build-distance-simd/artifacts/recall-build-distance-simd-table.log
```

Key result lines:

- `L=64 recall@k=0.9965 ndcg@k=0.9999 mean q-time=7.75 ms`
- `L=200 recall@k=0.9970 ndcg@k=0.9999 mean q-time=8.24 ms`
- `L=800 recall@k=0.9975 ndcg@k=0.9999 mean q-time=9.78 ms`

## Latency

Primary artifacts:

- `latency-build-distance-simd-after-restart-table.log`
- `latency-build-distance-simd-after-restart-cli.log`

Command:

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 bench latency --prefix task29c_phase_profile --profile ec_diskann --sweep 64,200,800 --iterations 500 --sample-backend-memory --log-file review/11108-task29d-build-distance-simd/artifacts/latency-build-distance-simd-after-restart-cli.log --log-output review/11108-task29d-build-distance-simd/artifacts/latency-build-distance-simd-after-restart-table.log
```

Key result lines:

- `L=64 mean=7.57 ms p50=7.45 ms p95=8.34 ms p99=9.81 ms hwm_peak_kb=61020`
- `L=200 mean=7.91 ms p50=7.80 ms p95=8.66 ms p99=10.6 ms hwm_peak_kb=61820`
- `L=800 mean=9.33 ms p50=9.19 ms p95=10.4 ms p99=13.1 ms hwm_peak_kb=62892`

Ignored audit artifacts:

- `latency-build-distance-simd-table.log`
- `latency-build-distance-simd-cli.log`

These were captured before the PG18 restart and showed inflated backend HWM
from previous backend lifetime state, so the after-restart run is the cited
latency/memory source.
