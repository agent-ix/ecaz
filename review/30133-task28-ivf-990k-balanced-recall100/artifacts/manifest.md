# Artifacts Manifest

## recall100_pqg8_990k_n128_w500_nprobe32_40.log

- head SHA: `d8be21cd`
- packet/topic: `30133-task28-ivf-990k-balanced-recall100`
- lane: Task 28 IVF 990k balanced recall@100 follow-up
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,40 --rerank-width 500 --force-index --log-output review/30133-task28-ivf-990k-balanced-recall100/artifacts/recall100_pqg8_990k_n128_w500_nprobe32_40.log`
- timestamp: 2026-04-28 18:43-19:14 PDT
- isolation: existing one-index-per-table 990k IVF surface from packet 30130
- key result lines:
  - `32 | 0.9360 | 0.9968 | 781.56 ms`
  - `40 | 0.9466 | 0.9975 | 928.39 ms`
