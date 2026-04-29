# Artifact Manifest

## latency_pqg8_990k_nprobe40_w250_i50.log

- head SHA: `8dbb274b`
- packet/topic: `30136-task28-ivf-990k-rerank-width`
- lane: Task 28 IVF 990k rerank-width latency sample
- fixture: existing DBPedia 990k IVF surface `task28_ivf_pqg990k_g8_n128`
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=250`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 50 --sweep 40 --rerank-width 250 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30136-task28-ivf-990k-rerank-width/artifacts/latency_pqg8_990k_nprobe40_w250_i50.log`
- timestamp: 2026-04-28T19:54:00-07:00
- surface: isolated one-index-per-table surface from packet 30130
- cache state: warm local PG18; no OS or Postgres cache drop
- key result lines:
  - `count 50`, `mean 856.6 ms`, `p50 857.5 ms`, `p95 974.1 ms`, `p99 1014.1 ms`, `rss_peak_kb 166216`, `hwm_peak_kb 166216`

## latency_pqg8_990k_nprobe40_w500_i50.log

- head SHA: `8dbb274b`
- packet/topic: `30136-task28-ivf-990k-rerank-width`
- lane: Task 28 IVF 990k rerank-width latency sample
- fixture: existing DBPedia 990k IVF surface `task28_ivf_pqg990k_g8_n128`
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 50 --sweep 40 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30136-task28-ivf-990k-rerank-width/artifacts/latency_pqg8_990k_nprobe40_w500_i50.log`
- timestamp: 2026-04-28T19:56:00-07:00
- surface: isolated one-index-per-table surface from packet 30130
- cache state: warm local PG18; no OS or Postgres cache drop
- key result lines:
  - `count 50`, `mean 896.2 ms`, `p50 891.6 ms`, `p95 1003.0 ms`, `p99 1043.6 ms`, `rss_peak_kb 166376`, `hwm_peak_kb 166376`
