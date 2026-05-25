# Task 59 AWS DiskANN Duplicate Expansion Fast Path Benchmark

- head SHA: `7d12eca92` on `diskann-aws-optimization`
- task bucket: benchmark packet `benchmarks/task59-aws-diskann-duplicate-expansion-fast-path/`
- suite config: `suite.json`
- suite: `task59-aws-diskann-duplicate-expansion-fast-path`
- cloud profile: `10k` (`m8g.large`, running Graviton cost-floor profile)
- run id: `20260524T194140Z`
- timestamp: `2026-05-24T19:42Z`
- storage format: `pq_fastscan`
- rerank mode: heap rerank, `rerank_budget=64`
- benchmark surface: shared retained Task 55 10k/100k tables
- command: `target/release/ecaz cloud bench --profile 10k --config benchmarks/task59-aws-diskann-duplicate-expansion-fast-path/suite.json --suite task59-diskann-fast-path --log-file benchmarks/task59-aws-diskann-duplicate-expansion-fast-path/artifacts/cloud-bench.log`

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
- `artifacts/cloud-install-1500c303d.log`: first install attempt; failed after build at extension drop due retained benchmark tables.
- `artifacts/cloud-install-diskann-aws-optimization.log`: branch install attempt; build/copy/restart succeeded, final extension drop failed because retained Task 55 tables depend on `ecvector`.

## Key Results

Against Task 55 optimized 100k latency, this slice was correctness-neutral but not a material latency win:

| dataset | list_size | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 64 | 0.9965 | 0.98 ms | 0.97 ms | 1.17 ms | 1.46 ms |
| 10k | 128 | 0.9965 | 1.29 ms | 1.29 ms | 1.60 ms | 1.78 ms |
| 10k | 200 | 0.9970 | 1.64 ms | 1.63 ms | 2.21 ms | 2.39 ms |
| 10k | 400 | 0.9970 | 2.40 ms | 2.43 ms | 3.13 ms | 3.44 ms |
| 10k | 800 | 0.9975 | 3.71 ms | 3.64 ms | 4.80 ms | 5.22 ms |
| 100k | 64 | 0.9165 | 1.77 ms | 1.74 ms | 2.25 ms | 2.51 ms |
| 100k | 128 | 0.9625 | 2.73 ms | 2.71 ms | 3.26 ms | 3.87 ms |
| 100k | 200 | 0.9745 | 3.57 ms | 3.55 ms | 4.44 ms | 4.72 ms |
| 100k | 400 | 0.9855 | 6.05 ms | 6.10 ms | 7.18 ms | 7.81 ms |
| 100k | 800 | 0.9865 | 10.7 ms | 10.8 ms | 12.8 ms | 13.4 ms |

Planner/config proof from `explain-100k-diskann-l800.log`:

- planner scan selection live: `planner_scan_enabled=t`
- `effective_list_size=800`
- `storage_format=pq_fastscan`
- `rerank_budget=64`

Conclusion: preserving the overflow flag avoids unnecessary duplicate-expansion reads, but this benchmark shows that path is capped by `rerank_budget=64`, not by `list_size`, so it is not the main high-`list_size` bottleneck.
