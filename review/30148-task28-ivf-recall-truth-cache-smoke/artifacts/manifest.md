# Artifacts Manifest

## recall_cache_miss.log

- head SHA: `5cfec355`
- packet/topic: `30148-task28-ivf-recall-truth-cache-smoke`
- lane: Task 28 IVF recall harness cache smoke
- fixture: `task28_ivf_pqg10k_g8_n128`, 3 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-dir review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/truth-cache --log-output review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/recall_cache_miss.log`
- timestamp: 2026-04-29 PDT
- isolation: existing one-index-per-table IVF surface
- key result lines:
  - `8 | 0.8667 | 0.9934 | 85.95 ms`
  - cache file written: `truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json`

## recall_cache_hit.log

- head SHA: `5cfec355`
- packet/topic: `30148-task28-ivf-recall-truth-cache-smoke`
- lane: Task 28 IVF recall harness cache smoke
- fixture: `task28_ivf_pqg10k_g8_n128`, 3 query cap
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-dir review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/truth-cache --log-output review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/recall_cache_hit.log`
- timestamp: 2026-04-29 PDT
- isolation: existing one-index-per-table IVF surface
- key result lines:
  - `8 | 0.8667 | 0.9934 | 59.98 ms`
  - cache file loaded: `truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json`

## truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json

- head SHA: `5cfec355`
- packet/topic: `30148-task28-ivf-recall-truth-cache-smoke`
- lane: Task 28 IVF recall harness cache smoke
- fixture: `task28_ivf_pqg10k_g8_n128`, 3 query cap
- command used to create: same as `recall_cache_miss.log`
- timestamp: 2026-04-29 PDT
- key descriptor:
  - `version=1`
  - `corpus_rows=10000`
  - `query_rows=3`
  - `dimensions=1536`
  - `k=10`
