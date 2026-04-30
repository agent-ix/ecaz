# Task 29a DiskANN Binary-Sidecar Prefilter Baseline

## Request

Review the Task 29a binary-sidecar scan prefilter result and landing
recommendation.

This packet measures commit `6491aeb60a6905ff546f117ce5d6d14d032059b4`
on local PG18 only. Commands used `ecaz-cli` / checked-in CLI surfaces,
not bare `psql`.

## Summary

The binary-sidecar prefilter resolves the Task 29 recall blocker.

- Fresh `task29a_sidecar_real10k` build with
  `graph_degree=32, build_list_size=100, alpha=1.2` completed in
  `503.10s` total: copy `4.27s`, encode `4.55s`, index build `492.13s`.
- Fresh DiskANN recall@10 is now above target at every requested list size:
  `0.9965` at L=64, `0.9970` at L=200, and `0.9975` at L=800.
- The known grouped-PQ miss for query `10001` is fixed. SQL now matches exact
  `10/10`, and IDs `9717` / `7782` are in the binary-sidecar frontier at ranks
  `25` / `47`, both inside `rerank_budget=64`.
- DiskANN sidecar at L=200: recall@10 `0.9970`, NDCG `0.9999`,
  mean query time `70.23 ms` on the fresh prefix. A repeated latency pass on
  the existing same-shape prefix measured p50 `66.6 ms`, p95 `71.7 ms`,
  p99 `73.5 ms`, backend HWM `70468 KiB`.
- Reference `ec_hnsw` at ef=200 on the same corpus: recall@10 `0.9700`,
  NDCG `0.9993`, mean query time `35.25 ms`; latency p50 `33.1 ms`,
  p95 `39.4 ms`, p99 `49.1 ms`, backend HWM `49028 KiB`.
- Fresh DiskANN index size is `4.7 MiB` / `494.0 B` per row. Reference HNSW
  index size is `13.0 MiB` / `1366.4 B` per row.
- Cache state for the local PG18 DB after the run was warm:
  `pg_stat_database` hit rate `99.56%`, `shared_buffers=128MB`,
  `effective_cache_size=4GB`.

## Recommendation

The landing blocker is closed. Keep the binary-sidecar prefilter as the
default `auto` path and do not spend more Task 29 time on grouped-PQ
prefilter tuning.

The next optimization should be scan latency, not recall:

1. Replace the linear scan/sort frontier in persisted greedy descent with a
   heap-based frontier and Vamana early-stop.
2. Then remove the redundant tuple read for the picked node during expansion.

These are the same latency issues called out in `review/11095`; they should
be a follow-up checkpoint because the quality fix is already isolated and
measured.

## Artifacts

See `artifacts/manifest.md` for command lines and packet-local raw logs.

Key logs:

- `artifacts/load-task29a-sidecar-real10k.log`
- `artifacts/recall-task29a-sidecar-fresh-table.log`
- `artifacts/sql-vs-memory-sidecar-auto.txt`
- `artifacts/frontier-binary-sidecar-q10001.txt`
- `artifacts/latency-sidecar-auto-table.log`
- `artifacts/storage-task29a-sidecar-fresh-cli.log`
- `artifacts/recall-ec-hnsw-reference-table.log`
- `artifacts/latency-ec-hnsw-reference-table.log`
- `artifacts/storage-ec-hnsw-reference-cli.log`
