# Artifact Manifest: Task 29 DiskANN Seeded Build Probe

Current branch head after reverting negative probe:
`55700d474cb5a2a8d62e8c15126da02103c14b93`

Measured code SHA: `2fb991ffcc29c19f8c8127cef2839dfac2bf48ab`
Packet: `11088-task29-diskann-seeded-build-probe`
Timestamp: `2026-04-29T18:37:34-07:00`

Lane: Task 29 DiskANN initial tuning
Fixture: local real-10k corpus from
`target/real-corpus/ec_hnsw_real_10k/`
Storage format: default `ecvector` / `ec_diskann`
Rerank mode: DiskANN V0 heap rerank path
Surface: isolated one-index-per-table prefix
`task29_diskann_seed_real10k` in database `task29_diskann_baseline`
Reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
Cache state: no cache flush; run after earlier Task 29 real-10k probes on the
same local PG18 cluster.

The measured code seeded the in-memory Vamana graph with deterministic random
out-neighbors before the existing two robust-prune passes. The measured commit
was reverted in `55700d47` because this probe did not improve recall or graph
hub concentration.

## `drop-seeded-prefix.log`

Command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "DROP TABLE IF EXISTS task29_diskann_seed_real10k_corpus CASCADE; DROP TABLE IF EXISTS task29_diskann_seed_real10k_queries CASCADE;" --log-output review/11088-task29-diskann-seeded-build-probe/artifacts/drop-seeded-prefix.log
```

Key result: prefix tables did not exist and were dropped/skipped cleanly.

## `load-diskann-seeded.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11088-task29-diskann-seeded-build-probe/artifacts/load-diskann-seeded.log corpus load --prefix task29_diskann_seed_real10k --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --allow-manifest-mismatch
```

Key result lines:

```text
copied corpus table task29_diskann_seed_real10k_corpus in 9.83s
encoded corpus table task29_diskann_seed_real10k_corpus in 4.95s
copied queries table task29_diskann_seed_real10k_queries in 213.92ms
built task29_diskann_seed_real10k_idx in 523.93s
completed prefix task29_diskann_seed_real10k in 555.60s
```

## `seeded-build-activity.log`

Command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "SELECT now(), state, wait_event_type, wait_event, left(query, 160) AS query FROM pg_stat_activity WHERE datname = 'task29_diskann_baseline' ORDER BY backend_start;" --log-output review/11088-task29-diskann-seeded-build-probe/artifacts/seeded-build-activity.log
```

Captured while checking the long-running build. By the time this query ran, the
loader backend had finished `CREATE INDEX` and autovacuum was analyzing the new
corpus table.

## `graph-diskann-seeded.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench diskann-graph --prefix task29_diskann_seed_real10k --log-output review/11088-task29-diskann-seeded-build-probe/artifacts/graph-diskann-seeded.log
```

Key result lines:

```text
reachable_live_node_count = 10000
unreachable_live_node_count = 0
neighbor_ref_count = 245015
dead_neighbor_ref_count = 0
invalid_neighbor_ref_count = 0
self_neighbor_ref_count = 0
duplicate_neighbor_ref_count = 0
unresolvable_neighbor_ref_count = 0
out degree: zero=0 min=6 avg=24.501500 p50=24 p95=32 p99=32 max=32
in degree: zero=0 min=4 avg=24.501500 p50=22 p95=43 p99=61 max=3480
```

## `recall-diskann-seeded-table.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench recall --prefix task29_diskann_seed_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11088-task29-diskann-seeded-build-probe/artifacts/recall-diskann-seeded-table.log
```

Key result lines:

```text
64  recall@10=0.9315  NDCG=0.9967  mean=60.26 ms
128 recall@10=0.9310  NDCG=0.9967  mean=71.04 ms
200 recall@10=0.9315  NDCG=0.9966  mean=84.90 ms
400 recall@10=0.9315  NDCG=0.9966  mean=133.17 ms
800 recall@10=0.9315  NDCG=0.9966  mean=278.34 ms
```

## `latency-diskann-seeded-table.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench latency --prefix task29_diskann_seed_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 200 --force-index --sample-backend-memory --log-output review/11088-task29-diskann-seeded-build-probe/artifacts/latency-diskann-seeded-table.log
```

Key result lines:

```text
64  mean=61.1 ms  p50=59.5 ms   p95=75.1 ms   p99=83.4 ms   hwm=118152 KiB
128 mean=68.7 ms  p50=68.7 ms   p95=73.7 ms   p99=76.5 ms   hwm=118632 KiB
200 mean=82.2 ms  p50=82.2 ms   p95=89.4 ms   p99=91.0 ms   hwm=118952 KiB
400 mean=127.8 ms p50=127.7 ms  p95=143.7 ms  p99=161.4 ms  hwm=118632 KiB
800 mean=277.1 ms p50=276.6 ms  p95=303.2 ms  p99=313.1 ms  hwm=118632 KiB
```

## `storage-diskann-seeded.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11088-task29-diskann-seeded-build-probe/artifacts/storage-diskann-seeded.log bench storage --prefix task29_diskann_seed_real10k
```

Key result:

```text
task29_diskann_seed_real10k_idx  ec_diskann  {graph_degree=32,build_list_size=100,alpha=1.2}  4.7 MiB  494.0 B/row
```
