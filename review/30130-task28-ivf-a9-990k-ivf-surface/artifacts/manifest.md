# Artifacts Manifest

## anchor_schema_inventory.sql / anchor_schema_inventory.log

- head SHA: `aabc71ea`
- packet/topic: `30130-task28-ivf-a9-990k-ivf-surface`
- lane: Task 28 IVF A9 990k anchor inventory
- fixture: existing DBPedia 990k anchor corpus and queries
- storage format: inventory
- rerank mode: inventory
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/anchor_schema_inventory.sql --raw --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/anchor_schema_inventory.log`
- timestamp: 2026-04-28 16:06 PDT
- isolation: shared anchor inventory before isolated IVF surface build
- key result lines:
  - `ec_hnsw_real_ann_benchmarks_anchor_corpus | 990000 | 75800576 | 72 MB`
  - `ec_hnsw_real_ann_benchmarks_anchor_queries | 10000 | 606208 | 592 kB`
  - `ec_hnsw_real_ann_benchmarks_anchor_m16_w8_idx | 1351688192 | 1289 MB`

## build_pqg8_990k_n128_w500.sql / build_pqg8_990k_n128_w500.log

- head SHA: `aabc71ea`
- packet/topic: `30130-task28-ivf-a9-990k-ivf-surface`
- lane: Task 28 IVF A9 990k selected surface build
- fixture: `task28_ivf_pqg990k_g8_n128`
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/build_pqg8_990k_n128_w500.sql --raw --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/build_pqg8_990k_n128_w500.log`
- timestamp: 2026-04-28 16:07-16:45 PDT
- isolation: one-index-per-table surface copied from the 990k anchor
- key result lines:
  - `SELECT 990000 | Time: 350927.311 ms (05:50.927)`
  - `SELECT 10000 | Time: 1910.149 ms (00:01.910)`
  - `CREATE INDEX | Time: 2033834.524 ms (33:53.835)`
  - `task28_ivf_pqg990k_g8_n128_idx | 185491456 | 177 MB | {nlists=128,nprobe=128,training_sample_rows=2000,quantizer=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`

## recall10_pqg8_990k_n128_w500.log

- head SHA: `aabc71ea`
- packet/topic: `30130-task28-ivf-a9-990k-ivf-surface`
- lane: Task 28 IVF A9 990k recall@10
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/recall10_pqg8_990k_n128_w500.log`
- timestamp: 2026-04-28 16:45-17:16 PDT
- isolation: one-index-per-table surface
- key result lines:
  - `48 | 0.9860 | 0.9990 | 1066.42 ms`

## latency_pqg8_990k_n128_w500.log

- head SHA: `aabc71ea`
- packet/topic: `30130-task28-ivf-a9-990k-ivf-surface`
- lane: Task 28 IVF A9 990k latency and memory
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 iterations
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/latency_pqg8_990k_n128_w500.log`
- timestamp: 2026-04-28 17:16-17:18 PDT
- isolation: one-index-per-table surface
- key result lines:
  - `48 | 100 | 1043.1 ms | 72.8 ms | 869.0 ms | 1029.2 ms | 1169.1 ms | 1224.4 ms | 1310.4 ms | 162636 | 162636 | 3889`

## recall100_pqg8_990k_n128_w500.log

- head SHA: `aabc71ea`
- packet/topic: `30130-task28-ivf-a9-990k-ivf-surface`
- lane: Task 28 IVF A9 990k recall@100
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/recall100_pqg8_990k_n128_w500.log`
- timestamp: 2026-04-28 17:18-17:48 PDT
- isolation: one-index-per-table surface
- key result lines:
  - `48 | 0.9509 | 0.9979 | 1088.61 ms`

## build progress logs

- head SHA: `aabc71ea`
- packet/topic: `30130-task28-ivf-a9-990k-ivf-surface`
- lane: Task 28 IVF A9 990k build progress diagnostics
- fixture: `task28_ivf_pqg990k_g8_n128`
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/build_progress.sql --raw --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/build_progress_00N.log`
- timestamp: 2026-04-28 16:09-16:31 PDT
- isolation: progress snapshots from a second PG18 connection while the build batch was running
- key result lines:
  - `CREATE TABLE ... SELECT * FROM ec_hnsw_real_ann_benchmarks_anchor_corpus`
  - `CREATE INDEX task28_ivf_pqg990k_g8_n128_idx`
  - `phase | building index`
