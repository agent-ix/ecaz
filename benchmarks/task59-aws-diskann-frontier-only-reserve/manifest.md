# Task 59 AWS DiskANN Frontier-Only Reserve

- head SHA measured: `ed7f025d4` (`Reserve only DiskANN frontier scratch in scans`)
- task: Task 59 (`plan/tasks/59-diskann-aws-graviton-tuning-1m-suite.md`)
- packet: `benchmarks/task59-aws-diskann-frontier-only-reserve/`
- hardware lane: AWS Graviton `10k` profile, `m7g.large`, retained-table run
- storage format: `pq_fastscan`
- table surface: retained isolated tables from Task 55:
  - `task55_real_10k_diskann_*`
  - `task55_real_100k_diskann_*`
- runner: `ecaz bench suite`
- suite config: `benchmarks/task59-aws-diskann-frontier-only-reserve/suite.json`
- command:
  - `target/release/ecaz cloud bench --profile 10k --config benchmarks/task59-aws-diskann-frontier-only-reserve/suite.json --log-file benchmarks/task59-aws-diskann-frontier-only-reserve/artifacts/cloud-bench.log`
- timestamp: 2026-05-24T21:11:23Z
- result: not promoted. Recall was unchanged, but 100k latency was neutral to worse versus the prior `no-visited-checks` checkpoint.

## Key Results

Initial 100k latency run:

| list_size | mean | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: |
| 64 | 1.73 ms | 1.70 ms | 2.16 ms | 2.37 ms |
| 128 | 2.62 ms | 2.61 ms | 3.12 ms | 3.71 ms |
| 200 | 3.55 ms | 3.53 ms | 4.32 ms | 4.63 ms |
| 400 | 5.95 ms | 6.02 ms | 7.17 ms | 7.52 ms |
| 800 | 10.5 ms | 10.6 ms | 12.6 ms | 13.4 ms |

100k latency rerun:

| list_size | mean | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: |
| 64 | 1.75 ms | 1.73 ms | 2.20 ms | 2.45 ms |
| 128 | 2.60 ms | 2.59 ms | 3.11 ms | 3.70 ms |
| 200 | 3.53 ms | 3.51 ms | 4.23 ms | 4.70 ms |
| 400 | 5.84 ms | 5.94 ms | 7.03 ms | 7.41 ms |
| 800 | 10.0 ms | 10.1 ms | 11.8 ms | 12.8 ms |

Prior promoted `no-visited-checks` checkpoint at 100k:

| list_size | mean | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: |
| 64 | 1.63 ms | 1.59 ms | 2.15 ms | 2.39 ms |
| 128 | 2.44 ms | 2.42 ms | 2.96 ms | 3.57 ms |
| 200 | 3.35 ms | 3.35 ms | 4.04 ms | 4.22 ms |
| 400 | 5.61 ms | 5.67 ms | 6.76 ms | 7.10 ms |
| 800 | 9.85 ms | 9.98 ms | 11.9 ms | 12.0 ms |

Recall matched the prior checkpoint exactly at 10k and 100k:

- 10k: `0.9965`, `0.9965`, `0.9970`, `0.9970`, `0.9975`
- 100k: `0.9165`, `0.9625`, `0.9745`, `0.9855`, `0.9865`

EXPLAIN/config proof from `artifacts/explain-100k-diskann-l800.log`:

- `effective_list_size=800`
- `storage_format=pq_fastscan`
- `rerank_budget=64`
- planner scan selection live

## Artifacts

- `artifacts/results.jsonl`
- `artifacts/rerun-100k-latency/results.jsonl`
- `artifacts/suite-manifest.json`
- `artifacts/suite-run.log`
- `artifacts/precheck-host.log`
- `artifacts/recall-10k-diskann-default.log`
- `artifacts/latency-10k-diskann-default.log`
- `artifacts/recall-100k-diskann-default.log`
- `artifacts/latency-100k-diskann-default.log`
- `artifacts/explain-100k-diskann-l800.log`
