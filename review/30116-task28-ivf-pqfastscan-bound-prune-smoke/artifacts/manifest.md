# Artifact Manifest

## recall10_pqg8_100k_n128_w500_bound.log

- head SHA: `f47678a2`
- packet/topic: `30116-task28-ivf-pqfastscan-bound-prune-smoke`
- lane: Task 28 IVF A7 score-bound pruning smoke
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- nprobe sweep: `48`
- rerank mode: `heap_f32`
- rerank width: `500`
- k: `10`
- timestamp: `2026-04-28T12:54:16-07:00`
- surface: shared-table 100k ec_ivf benchmark surface created by earlier Task 28 packets
- command:
  `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30116-task28-ivf-pqfastscan-bound-prune-smoke/artifacts/recall10_pqg8_100k_n128_w500_bound.log`
- key result:
  `48 | recall@10 0.9920 | NDCG@10 0.9997 | mean q-time 191.60 ms`

## latency_pqg8_100k_n128_w500_bound.log

- head SHA: `f47678a2`
- packet/topic: `30116-task28-ivf-pqfastscan-bound-prune-smoke`
- lane: Task 28 IVF A7 score-bound pruning smoke
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- nprobe sweep: `48`
- rerank mode: `heap_f32`
- rerank width: `500`
- k: `10`
- timestamp: `2026-04-28T12:54:16-07:00`
- surface: shared-table 100k ec_ivf benchmark surface created by earlier Task 28 packets
- command:
  `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30116-task28-ivf-pqfastscan-bound-prune-smoke/artifacts/latency_pqg8_100k_n128_w500_bound.log`
- key result:
  `48 | count 100 | mean 175.4 ms | p50 173.1 ms | p95 204.9 ms | p99 210.5 ms | HWM 156692 kB`
