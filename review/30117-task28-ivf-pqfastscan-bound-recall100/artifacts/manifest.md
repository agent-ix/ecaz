# Artifact Manifest

## recall100_pqg8_100k_n128_w500_bound.log

- head SHA: `05995a3a`
- packet/topic: `30117-task28-ivf-pqfastscan-bound-recall100`
- lane: Task 28 IVF A9/A10 recall@100 follow-up after A7 pruning
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- nprobe sweep: `48`
- rerank mode: `heap_f32`
- rerank width: `500`
- k: `100`
- timestamp: `2026-04-28T12:58:39-07:00`
- surface: shared-table 100k ec_ivf benchmark surface created by earlier Task 28 packets
- command:
  `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30117-task28-ivf-pqfastscan-bound-recall100/artifacts/recall100_pqg8_100k_n128_w500_bound.log`
- key result:
  `48 | recall@100 0.9552 | NDCG@100 0.9983 | mean q-time 207.16 ms`
