# Artifact Manifest

## recall10_pqg8_100k_n128_w500_current.log

- head SHA: `d234d8da`
- packet/topic: `30111-task28-ivf-a9-100k-current-head`
- lane: Task 28 IVF A9 100k current-head scan rerun
- fixture: existing isolated `task28_ivf_pqg100k_g8_n128` 100k corpus, 100 queries
- storage format / quantizer: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48,56,64 --rerank-width 500 --force-index --log-output review/30111-task28-ivf-a9-100k-current-head/artifacts/recall10_pqg8_100k_n128_w500_current.log`
- timestamp: `2026-04-28T12:05:20-07:00`
- isolated/shared surface: isolated one-index-per-table surface
- key cited result lines:
  - `nprobe=48`: `recall@10=0.9920`, `ndcg@10=0.9997`, `mean q-time=246.11 ms`
  - `nprobe=56`: `recall@10=0.9930`, `ndcg@10=0.9997`, `mean q-time=276.19 ms`
  - `nprobe=64`: `recall@10=0.9940`, `ndcg@10=0.9997`, `mean q-time=307.45 ms`

## recall100_pqg8_100k_n128_w500_current.log

- head SHA: `d234d8da`
- packet/topic: `30111-task28-ivf-a9-100k-current-head`
- lane: Task 28 IVF A9 100k current-head scan rerun
- fixture: existing isolated `task28_ivf_pqg100k_g8_n128` 100k corpus, 100 queries
- storage format / quantizer: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48,56,64 --rerank-width 500 --force-index --log-output review/30111-task28-ivf-a9-100k-current-head/artifacts/recall100_pqg8_100k_n128_w500_current.log`
- timestamp: `2026-04-28T12:05:20-07:00`
- isolated/shared surface: isolated one-index-per-table surface
- key cited result lines:
  - `nprobe=48`: `recall@100=0.9552`, `ndcg@100=0.9983`, `mean q-time=280.61 ms`
  - `nprobe=56`: `recall@100=0.9584`, `ndcg@100=0.9985`, `mean q-time=317.03 ms`
  - `nprobe=64`: `recall@100=0.9619`, `ndcg@100=0.9987`, `mean q-time=344.32 ms`

## latency_pqg8_100k_n128_w500_current.log

- head SHA: `d234d8da`
- packet/topic: `30111-task28-ivf-a9-100k-current-head`
- lane: Task 28 IVF A9 100k current-head scan rerun
- fixture: existing isolated `task28_ivf_pqg100k_g8_n128` 100k corpus, 100 queries
- storage format / quantizer: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48,56,64 --rerank-width 500 --force-index --log-output review/30111-task28-ivf-a9-100k-current-head/artifacts/latency_pqg8_100k_n128_w500_current.log`
- timestamp: `2026-04-28T12:05:20-07:00`
- isolated/shared surface: isolated one-index-per-table surface
- key cited result lines:
  - `nprobe=48`: `count=100`, `mean=245.0 ms`, `p50=242.9 ms`, `p95=270.8 ms`, `p99=294.0 ms`
  - `nprobe=56`: `count=100`, `mean=282.3 ms`, `p50=281.2 ms`, `p95=317.3 ms`, `p99=329.3 ms`
  - `nprobe=64`: `count=100`, `mean=310.2 ms`, `p50=307.1 ms`, `p95=352.2 ms`, `p99=367.9 ms`
