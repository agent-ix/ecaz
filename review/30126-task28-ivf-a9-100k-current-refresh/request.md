# Task 28 IVF A9 100k Current-Head Refresh

## Scope

This packet refreshes the selected 100k IVF operating point at current head after the A3 adjacent-page reuse work.

Fixture:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=500`
- PG18 local database

The existing 100k surface was reused. Commit `4ed20913` changes insert/vacuum page reuse and does not change scan scoring or build layout for this already-built surface, so this packet refreshes recall/latency/size without rebuilding.

## Result

| metric | value |
|---|---:|
| build time | 216788.531 ms |
| index size | 19,791,872 bytes |
| recall@10 | 0.9920 |
| NDCG@10 | 0.9997 |
| recall@100 | 0.9552 |
| NDCG@100 | 0.9983 |
| latency p50 | 169.3 ms |
| latency p95 | 191.2 ms |
| latency p99 | 194.4 ms |
| memory HWM | 153816 kB |

Cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop.

Build time is carried forward from packet 30119's fresh rebuild of this same selected surface. Current-head size, recall, latency, memory, and reloptions are packet-local in `artifacts/`.

## Interpretation

This is the cleanest current-head 100k IVF selected-point packet so far. Recall is unchanged from packet 30119, while local warm latency/HWM improved modestly:

- packet 30119: p50/p95/p99 `173.4/225.4/242.9 ms`, HWM `157108 kB`
- packet 30126: p50/p95/p99 `169.3/191.2/194.4 ms`, HWM `153816 kB`

This closes the current-head 100k IVF selected-point portion of A9. It does not close the full A9 wording for 1M and matched HNSW comparison.

## Validation

- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/recall10_pqg8_100k_n128_w500_current.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/recall100_pqg8_100k_n128_w500_current.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/latency_pqg8_100k_n128_w500_current.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT c.relname, pg_relation_size(c.oid) AS index_bytes, pg_size_pretty(pg_relation_size(c.oid)) AS index_size, c.reloptions FROM pg_class c WHERE c.relname = 'task28_ivf_pqg100k_g8_n128_idx'; SELECT pg_relation_size('task28_ivf_pqg100k_g8_n128_corpus'::regclass) AS corpus_heap_bytes, pg_total_relation_size('task28_ivf_pqg100k_g8_n128_corpus'::regclass) AS corpus_total_bytes;" --raw --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/size_cache_pqg8_100k_n128_w500_current.log`

## Artifacts

- `artifacts/recall10_pqg8_100k_n128_w500_current.log`
- `artifacts/recall100_pqg8_100k_n128_w500_current.log`
- `artifacts/latency_pqg8_100k_n128_w500_current.log`
- `artifacts/size_cache_pqg8_100k_n128_w500_current.log`
- `artifacts/manifest.md`
