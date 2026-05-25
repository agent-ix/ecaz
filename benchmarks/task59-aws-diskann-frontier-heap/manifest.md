# Task 59 AWS DiskANN Frontier Heap Benchmark

- head SHA: `d37d086720b4833677dabdeb25d7eb27f2e76904`
- task bucket: benchmark packet `benchmarks/task59-aws-diskann-frontier-heap/`
- suite config: `suite.json`
- suite: `task59-aws-diskann-frontier-heap`
- cloud profile: `10k` (`m8g.large`, running Graviton cost-floor profile)
- run id: `20260524T202501Z`
- timestamp: `2026-05-24T20:25Z`
- storage format: `pq_fastscan`
- rerank mode: heap rerank, `rerank_budget=64`
- benchmark surface: shared retained Task 55 10k/100k tables
- command: `target/release/ecaz cloud bench --profile 10k --config benchmarks/task59-aws-diskann-frontier-heap/suite.json --suite task59-diskann-frontier-heap --log-file benchmarks/task59-aws-diskann-frontier-heap/artifacts/cloud-bench.log`

## Artifacts

- `artifacts/suite-run.log`: remote `ecaz bench suite run` log.
- `artifacts/suite-manifest.json`: suite manifest.
- `artifacts/results.jsonl`: normalized suite rows.
- `artifacts/precheck-host.log`: PostgreSQL config and CPU precheck.
- `artifacts/recall-10k-diskann-default.log`
- `artifacts/latency-10k-diskann-default.log`
- `artifacts/recall-100k-diskann-default.log`
- `artifacts/latency-100k-diskann-default.log`
- `artifacts/explain-100k-diskann-l800.log`

## Key Results

The frontier heap change preserved recall and produced a modest high-`list_size` improvement.

| dataset | list_size | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 64 | 0.9965 | 0.93 ms | 0.92 ms | 1.10 ms | 1.36 ms |
| 10k | 128 | 0.9965 | 1.25 ms | 1.24 ms | 1.57 ms | 1.73 ms |
| 10k | 200 | 0.9970 | 1.56 ms | 1.56 ms | 2.04 ms | 2.28 ms |
| 10k | 400 | 0.9970 | 2.23 ms | 2.22 ms | 2.87 ms | 3.23 ms |
| 10k | 800 | 0.9975 | 3.42 ms | 3.35 ms | 4.38 ms | 4.60 ms |
| 100k | 64 | 0.9165 | 1.76 ms | 1.72 ms | 2.25 ms | 2.46 ms |
| 100k | 128 | 0.9625 | 2.54 ms | 2.52 ms | 3.06 ms | 3.70 ms |
| 100k | 200 | 0.9745 | 3.45 ms | 3.43 ms | 4.18 ms | 4.53 ms |
| 100k | 400 | 0.9855 | 5.90 ms | 5.96 ms | 7.20 ms | 7.50 ms |
| 100k | 800 | 0.9865 | 10.2 ms | 10.2 ms | 12.0 ms | 12.9 ms |

Task 55 optimized comparison at 100k:

- `list_size=128`: mean `2.60 -> 2.54 ms`, p95 `3.18 -> 3.06 ms`
- `list_size=200`: mean `3.49 -> 3.45 ms`, p95 `4.27 -> 4.18 ms`
- `list_size=400`: mean `5.88 -> 5.90 ms`, p95 `7.02 -> 7.20 ms`
- `list_size=800`: mean `10.6 -> 10.2 ms`, p95 `12.4 -> 12.0 ms`

Planner/config proof from `explain-100k-diskann-l800.log`:

- planner scan selection live: `planner_scan_enabled=t`
- `effective_list_size=800`
- `storage_format=pq_fastscan`
- `rerank_budget=64`

Conclusion: this is a small but directionally useful traversal-state cleanup. It does not satisfy Task 59's larger optimization target by itself; 1M/profile tuning is still required.
