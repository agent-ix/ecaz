# Artifacts Manifest

## recall10_pqg8_100k_n128_w500_current.log

- head SHA: `2050e60d`
- packet/topic: `30126-task28-ivf-a9-100k-current-refresh`
- lane: Task 28 IVF A9 current-head 100k selected point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/recall10_pqg8_100k_n128_w500_current.log`
- timestamp: 2026-04-28 America/Los_Angeles
- isolation: shared-table 100k surface
- cache state: warm local development run; no explicit cache drop
- key result lines:
  - `48 | recall@10 0.9920 | NDCG@10 0.9997 | mean q-time 194.41 ms`

## recall100_pqg8_100k_n128_w500_current.log

- head SHA: `2050e60d`
- packet/topic: `30126-task28-ivf-a9-100k-current-refresh`
- lane: Task 28 IVF A9 current-head 100k selected point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/recall100_pqg8_100k_n128_w500_current.log`
- timestamp: 2026-04-28 America/Los_Angeles
- isolation: shared-table 100k surface
- cache state: warm local development run; no explicit cache drop
- key result lines:
  - `48 | recall@100 0.9552 | NDCG@100 0.9983 | mean q-time 211.28 ms`

## latency_pqg8_100k_n128_w500_current.log

- head SHA: `2050e60d`
- packet/topic: `30126-task28-ivf-a9-100k-current-refresh`
- lane: Task 28 IVF A9 current-head 100k selected point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/latency_pqg8_100k_n128_w500_current.log`
- timestamp: 2026-04-28 America/Los_Angeles
- isolation: shared-table 100k surface
- cache state: warm local development run; no explicit cache drop
- key result lines:
  - `48 | count 100 | mean 169.4 ms | p50 169.3 ms | p95 191.2 ms | p99 194.4 ms | rss_peak_kb 153816 | hwm_peak_kb 153816`

## size_cache_pqg8_100k_n128_w500_current.log

- head SHA: `2050e60d`
- packet/topic: `30126-task28-ivf-a9-100k-current-refresh`
- lane: Task 28 IVF A9 current-head 100k selected point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT c.relname, pg_relation_size(c.oid) AS index_bytes, pg_size_pretty(pg_relation_size(c.oid)) AS index_size, c.reloptions FROM pg_class c WHERE c.relname = 'task28_ivf_pqg100k_g8_n128_idx'; SELECT pg_relation_size('task28_ivf_pqg100k_g8_n128_corpus'::regclass) AS corpus_heap_bytes, pg_total_relation_size('task28_ivf_pqg100k_g8_n128_corpus'::regclass) AS corpus_total_bytes;" --raw --log-output review/30126-task28-ivf-a9-100k-current-refresh/artifacts/size_cache_pqg8_100k_n128_w500_current.log`
- timestamp: 2026-04-28 America/Los_Angeles
- isolation: shared-table 100k surface
- cache state: warm local development run; no explicit cache drop
- key result lines:
  - `task28_ivf_pqg100k_g8_n128_idx | 19791872 | 19 MB | {nlists=128,nprobe=128,training_sample_rows=2000,quantizer=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`
  - `corpus_heap_bytes 7675904 | corpus_total_bytes 1686732800`
