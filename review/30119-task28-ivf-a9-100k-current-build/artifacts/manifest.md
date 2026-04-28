# Artifact Manifest

## build_pqg8_100k_n128_w500_current.log

- head SHA: `19436385`
- packet/topic: `30119-task28-ivf-a9-100k-current-build`
- lane: Task 28 IVF A9 fresh 100k selected operating point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- rerank mode: `heap_f32`
- rerank width: `500`
- timestamp: `2026-04-28T13:15:07-07:00`
- surface: shared-table 100k ec_ivf surface rebuilt by this packet
- cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop
- command:
  `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30119-task28-ivf-a9-100k-current-build/artifacts/build_pqg8_100k_n128_w500_current.sql --raw --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/build_pqg8_100k_n128_w500_current.log`
- key result:
  `CREATE INDEX | 216788.531 ms | index_bytes 19791872 | 19 MB`

## recall10_pqg8_100k_n128_w500_fresh.log

- head SHA: `19436385`
- packet/topic: `30119-task28-ivf-a9-100k-current-build`
- lane: Task 28 IVF A9 fresh 100k selected operating point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- nprobe sweep: `48`
- rerank mode: `heap_f32`
- rerank width: `500`
- k: `10`
- timestamp: `2026-04-28T13:15:07-07:00`
- surface: shared-table 100k ec_ivf surface rebuilt by this packet
- cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop
- command:
  `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/recall10_pqg8_100k_n128_w500_fresh.log`
- key result:
  `48 | recall@10 0.9920 | NDCG@10 0.9997 | mean q-time 171.23 ms`

## recall100_pqg8_100k_n128_w500_fresh.log

- head SHA: `19436385`
- packet/topic: `30119-task28-ivf-a9-100k-current-build`
- lane: Task 28 IVF A9 fresh 100k selected operating point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- nprobe sweep: `48`
- rerank mode: `heap_f32`
- rerank width: `500`
- k: `100`
- timestamp: `2026-04-28T13:15:07-07:00`
- surface: shared-table 100k ec_ivf surface rebuilt by this packet
- cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop
- command:
  `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/recall100_pqg8_100k_n128_w500_fresh.log`
- key result:
  `48 | recall@100 0.9552 | NDCG@100 0.9983 | mean q-time 210.80 ms`

## latency_pqg8_100k_n128_w500_fresh.log

- head SHA: `19436385`
- packet/topic: `30119-task28-ivf-a9-100k-current-build`
- lane: Task 28 IVF A9 fresh 100k selected operating point
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- nprobe sweep: `48`
- rerank mode: `heap_f32`
- rerank width: `500`
- k: `10`
- timestamp: `2026-04-28T13:15:07-07:00`
- surface: shared-table 100k ec_ivf surface rebuilt by this packet
- cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop
- command:
  `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30119-task28-ivf-a9-100k-current-build/artifacts/latency_pqg8_100k_n128_w500_fresh.log`
- key result:
  `48 | count 100 | mean 177.1 ms | p50 173.4 ms | p95 225.4 ms | p99 242.9 ms | HWM 157108 kB`
