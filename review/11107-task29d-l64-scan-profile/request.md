# Task 29d L64 Scan Profile

Status: ready for review
Branch: `task29-diskann-initial-tuning`
Head: `b2bf4f6992a4895ede6ad4b50131102317db145f`

## Question

Task 29d asked whether the release-mode L=64 DiskANN scan latency gap could be
closed before landing. Packet `11105` showed pgvectorscale at 3.56 ms and
ec_diskann above that at L=64. This packet profiles the current ec_diskann
L=64 path and tests the most obvious safe scan-time lever: exact heap rerank
budget.

## Result

No production change is recommended from this slice.

After rebuilding the test index with the current non-experimental extension,
the valid current-head L=64 scan latency was:

| Setting | Recall@10 | NDCG@10 | Mean query | p50 | p95 | p99 | HWM |
|---|---:|---:|---:|---:|---:|---:|---:|
| `rerank_budget=64` current default | prior sweep 0.9965 | prior sweep 0.9999 | 7.82 ms | 7.70 ms | 8.46 ms | 11.8 ms | 61504 KiB |

System `perf record` is blocked on this machine by
`perf_event_paranoid=2`, so this packet used `EXPLAIN (ANALYZE, BUFFERS)` plus
rerank-window A/B.

The rerank A/B shows exact heap rerank is a major L=64 cost, but reducing it
far enough to hit pgvectorscale-like latency loses too much recall:

| `rerank_budget` | Recall@10 | NDCG@10 | Recall mean query | Latency mean | p50 | p95 | p99 | HWM |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 10 | 0.8600 | 0.9962 | 3.38 ms | 3.43 ms | 3.37 ms | 3.95 ms | 4.53 ms | 51424 KiB |
| 32 | 0.9880 | 0.9998 | 5.50 ms | 5.26 ms | 5.18 ms | 5.85 ms | 6.58 ms | 59744 KiB |
| 48 | 0.9955 | 0.9999 | 6.79 ms | 6.58 ms | 6.46 ms | 7.45 ms | 8.50 ms | 60704 KiB |
| 52 | 0.9955 | 0.9999 | 7.20 ms | 6.89 ms | 6.74 ms | 7.97 ms | 9.81 ms | 60864 KiB |
| 56 | 0.9960 | 0.9999 | 7.42 ms | 7.16 ms | 7.05 ms | 8.06 ms | 9.93 ms | 61184 KiB |

The first tested value that meets the established recall floor is
`rerank_budget=56`, but its mean latency is still 7.16 ms. That misses the
sub-6 ms stop target from Task 29d, so the correct action is to document the
residual instead of weakening the default.

The index was restored to:

`{graph_degree=32,build_list_size=100,alpha=1.2,rerank_budget=64,top_k=10}`

## Notes

- The first L=64 latency run in this artifact directory was captured before
  rebuilding the index after the heap-frontier A/B. It is retained only as an
  audit trail and is not cited as a result.
- `EXPLAIN (ANALYZE, BUFFERS)` on representative queries showed warm-cache
  index scans around 10.937 ms and 15.703 ms with about 984 to 985 shared
  buffer hits, supporting the conclusion that the residual is not cold-cache
  I/O.
- This packet does not alter code or reloptions defaults.

## Artifacts

See `artifacts/manifest.md` for commands and packet-local source logs.
