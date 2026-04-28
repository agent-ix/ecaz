# Artifacts Manifest

## a9_local_inventory.sql / a9_local_inventory.log

- head SHA: `9d18ae66`
- packet/topic: `30128-task28-ivf-a9-remaining-inventory`
- lane: Task 28 IVF A9 remaining local inventory
- fixture: local PG18 corpus/index inventory for 100k IVF/HNSW and 990k anchor surfaces
- storage format: mixed inventory
- rerank mode: mixed inventory
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30128-task28-ivf-a9-remaining-inventory/artifacts/a9_local_inventory.sql --raw --log-output review/30128-task28-ivf-a9-remaining-inventory/artifacts/a9_local_inventory.log`
- timestamp: 2026-04-28 America/Los_Angeles
- isolation: shared local development database inventory
- key result lines:
  - `task28_a9_100k_ivf_corpus | 100000 | 7864320 | 1686921216`
  - `task28_a9_100k_hnsw_corpus | 100000 | 7864320 | 1667129344`
  - `ec_hnsw_real_ann_benchmarks_anchor_corpus | 990000 | 75800576 | 17853349888`
  - `task28_a9_100k_ivf_idx | 100000 | 19791872 | 19 MB`
  - `ec_hnsw_real_ann_benchmarks_anchor_m16_w8_idx | 990000 | 1351688192 | 1289 MB`
