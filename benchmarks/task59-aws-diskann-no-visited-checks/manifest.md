# Task 59 AWS DiskANN No Visited Checks Benchmark

- head SHA: `af9e874f7c0ad5b979e9d649083f871fe72f591b`
- task bucket: benchmark packet `benchmarks/task59-aws-diskann-no-visited-checks/`
- suite config: `suite.json`
- suite: `task59-aws-diskann-no-visited-checks`
- cloud profile: `10k` (`m8g.large`, running Graviton cost-floor profile)
- run id: `20260524T205300Z`
- timestamp: `2026-05-24T20:53Z`
- storage format: `pq_fastscan`
- rerank mode: heap rerank, `rerank_budget=64`
- benchmark surface: shared retained Task 55 10k/100k tables
- command: `target/release/ecaz cloud bench --profile 10k --config benchmarks/task59-aws-diskann-no-visited-checks/suite.json --suite task59-diskann-no-visited-checks --log-file benchmarks/task59-aws-diskann-no-visited-checks/artifacts/cloud-bench.log`

## Artifacts

- `artifacts/suite-run.log`: remote `ecaz bench suite run` log.
- `artifacts/suite-manifest.json`: suite manifest.
- `artifacts/results.jsonl`: normalized suite rows.
- `artifacts/precheck-host.log`: PostgreSQL config and CPU precheck.
- `artifacts/cloud-install.log`: clean install using `--skip-extension-recreate`.
- `artifacts/recall-10k-diskann-default.log`
- `artifacts/latency-10k-diskann-default.log`
- `artifacts/recall-100k-diskann-default.log`
- `artifacts/latency-100k-diskann-default.log`
- `artifacts/explain-100k-diskann-l800.log`

## Key Results

The no-visited-checks scan cleanup preserved recall and improved the high-`list_size`
curve on 100k.

| dataset | list_size | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 64 | 0.9965 | 0.96 ms | 0.95 ms | 1.15 ms | 1.32 ms |
| 10k | 128 | 0.9965 | 1.26 ms | 1.25 ms | 1.60 ms | 1.84 ms |
| 10k | 200 | 0.9970 | 1.54 ms | 1.53 ms | 2.02 ms | 2.24 ms |
| 10k | 400 | 0.9970 | 2.26 ms | 2.28 ms | 2.88 ms | 3.28 ms |
| 10k | 800 | 0.9975 | 3.50 ms | 3.44 ms | 4.50 ms | 4.87 ms |
| 100k | 64 | 0.9165 | 1.63 ms | 1.59 ms | 2.15 ms | 2.39 ms |
| 100k | 128 | 0.9625 | 2.44 ms | 2.42 ms | 2.96 ms | 3.57 ms |
| 100k | 200 | 0.9745 | 3.35 ms | 3.35 ms | 4.04 ms | 4.22 ms |
| 100k | 400 | 0.9855 | 5.61 ms | 5.67 ms | 6.76 ms | 7.10 ms |
| 100k | 800 | 0.9865 | 9.85 ms | 9.98 ms | 11.9 ms | 12.0 ms |

Task 55 optimized comparison at 100k:

- `list_size=64`: mean `1.72 -> 1.63 ms`, p95 `2.21 -> 2.15 ms`
- `list_size=128`: mean `2.60 -> 2.44 ms`, p95 `3.18 -> 2.96 ms`
- `list_size=200`: mean `3.49 -> 3.35 ms`, p95 `4.27 -> 4.04 ms`
- `list_size=400`: mean `5.88 -> 5.61 ms`, p95 `7.02 -> 6.76 ms`
- `list_size=800`: mean `10.6 -> 9.85 ms`, p95 `12.4 -> 11.9 ms`

Planner/config proof from `explain-100k-diskann-l800.log`:

- planner scan selection live: `planner_scan_enabled=t`
- `effective_list_size=800`
- `storage_format=pq_fastscan`
- `rerank_budget=64`

Conclusion: this is the strongest Task 59 scan-loop cleanup so far. It is still not
the final Task 59 suite; 50k/1M and Graviton profile selection remain open.
