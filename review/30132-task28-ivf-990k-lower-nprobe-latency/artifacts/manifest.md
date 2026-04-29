# Artifacts Manifest

## latency_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log

- head SHA: `36d13f35`
- packet/topic: `30132-task28-ivf-990k-lower-nprobe-latency`
- lane: Task 28 IVF 990k lower-nprobe latency frontier
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 iterations per point
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 16,24,32,40,48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30132-task28-ivf-990k-lower-nprobe-latency/artifacts/latency_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log`
- timestamp: 2026-04-28 17:55-18:02 PDT
- isolation: existing one-index-per-table 990k IVF surface from packet 30130
- key result lines:
  - `16 | 100 | 434.8 ms | 53.9 ms | 327.4 ms | 432.3 ms | 522.8 ms | 575.0 ms | 605.4 ms | 157340 | 157340 | 1627`
  - `24 | 100 | 591.4 ms | 62.4 ms | 454.1 ms | 580.5 ms | 678.6 ms | 730.7 ms | 913.5 ms | 166340 | 166340 | 2209`
  - `32 | 100 | 740.6 ms | 61.4 ms | 583.2 ms | 740.1 ms | 833.8 ms | 876.6 ms | 956.6 ms | 162588 | 162588 | 2766`
  - `40 | 100 | 887.8 ms | 67.5 ms | 725.6 ms | 884.2 ms | 994.8 ms | 1036.2 ms | 1054.9 ms | 162636 | 162636 | 3321`
  - `48 | 100 | 1047.1 ms | 74.0 ms | 864.8 ms | 1042.8 ms | 1179.9 ms | 1229.2 ms | 1242.0 ms | 162664 | 162664 | 3920`

## recall10_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log

- head SHA: `36d13f35`
- packet/topic: `30132-task28-ivf-990k-lower-nprobe-latency`
- lane: Task 28 IVF 990k lower-nprobe recall@10 frontier
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 16,24,32,40,48 --rerank-width 500 --force-index --log-output review/30132-task28-ivf-990k-lower-nprobe-latency/artifacts/recall10_pqg8_990k_n128_w500_nprobe16_24_32_40_48.log`
- timestamp: 2026-04-28 18:03-18:41 PDT
- isolation: existing one-index-per-table 990k IVF surface from packet 30130
- key result lines:
  - `16 | 0.9380 | 0.9959 | 448.30 ms`
  - `24 | 0.9640 | 0.9976 | 590.08 ms`
  - `32 | 0.9750 | 0.9984 | 741.11 ms`
  - `40 | 0.9810 | 0.9987 | 897.16 ms`
  - `48 | 0.9860 | 0.9990 | 1042.06 ms`
