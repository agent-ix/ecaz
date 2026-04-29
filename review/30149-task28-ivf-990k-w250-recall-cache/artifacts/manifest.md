# Artifacts Manifest

## recall_cache_file_hit_10k.log

- head SHA: `101e31fd`
- packet/topic: `30149-task28-ivf-990k-w250-recall-cache`
- lane: Task 28 IVF recall harness cache-file smoke
- fixture: `task28_ivf_pqg10k_g8_n128`, 3 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-file review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/truth-cache/truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json --log-output review/30149-task28-ivf-990k-w250-recall-cache/artifacts/recall_cache_file_hit_10k.log`
- timestamp: 2026-04-29 PDT
- isolation: existing one-index-per-table IVF surface
- key result lines:
  - `8 | 0.8667 | 0.9934 | 70.69 ms`

## aborted 990k width-250 recall attempt

- head SHA: `c0147528`
- packet/topic: `30149-task28-ivf-990k-w250-recall-cache`
- lane: Task 28 IVF 990k width-250 recall follow-up
- fixture: `task28_ivf_pqg990k_g8_n128`, 100 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=250`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 40 --rerank-width 250 --force-index --truth-cache-dir review/30149-task28-ivf-990k-w250-recall-cache/artifacts/truth-cache --log-output review/30149-task28-ivf-990k-w250-recall-cache/artifacts/recall100_pqg8_990k_n128_w250_nprobe40.log`
- timestamp: 2026-04-29 PDT
- isolation: existing one-index-per-table IVF surface
- outcome: stopped after roughly 22 minutes while still fetching
  `task28_ivf_pqg990k_g8_n128_corpus`; no recall log was written
