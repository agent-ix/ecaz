# Task 59 AWS Graviton DiskANN Final Suite

- head SHA at packet creation: `130b70592032cd91ab7c204ac14ee63a88fdda5c`
- scan-path code SHA covered by measurements: `cf80e62dc` and later branch heads through `130b70592`; no `src/am/ec_diskann` changes landed after the measured scan-loop optimizations
- packet path: `benchmarks/task59-aws-diskann-final-graviton-suite`
- timestamp: `2026-05-25T02:47:23Z`
- profile: `10k-medium`, AWS Graviton `m8g.2xlarge`, 8 vCPU, 32 GiB RAM, 100 GiB gp3
- database host: `10.42.1.58`, instance `i-0d46840c00f93dbb6`
- bucket: `ecaz-cloud-10k-medium-a02e4aea`
- PostgreSQL: 18.3, `shared_buffers=128MB`, `work_mem=64MB`, `maintenance_work_mem=512MB`, `effective_cache_size=4GB`
- storage format: `pq_fastscan`
- rerank mode: default DiskANN scan path, `rerank_budget=64`
- isolated one-index-per-table: yes, per scale

## Runs

### 10k/50k/100k final suite

- config: `suite.json`
- command: `target/release/ecaz cloud bench --profile 10k-medium --config benchmarks/task59-aws-diskann-final-graviton-suite/suite.json --log-file benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/cloud-bench-10k-medium-final.log`
- artifact root: `artifacts/s3-final-224632`
- result: setup, load, recall, latency, storage, and explain succeeded for 10k, 50k, and 100k; the original 1M load failed because the retained 1M staging manifest was single-TSV, not chunked.
- key status source: `artifacts/s3-final-224632/suite-manifest.json`
- S3 copy source: `s3://ecaz-cloud-10k-medium-a02e4aea/bench-artifacts/smoke/20260524T224632Z/`

### 1M resume suite

- config: `suite-1m-resume.json`
- command: `target/release/ecaz cloud bench --profile 10k-medium --config benchmarks/task59-aws-diskann-final-graviton-suite/suite-1m-resume.json --log-file benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/cloud-bench-10k-medium-1m-resume-rerun.log`
- artifact root: `artifacts/one-million-resume`
- result: load, recall, latency, storage, and explain succeeded for 1M after dropping the partial 1M table and removing duplicate fetched parquet directories.
- key status source: `artifacts/one-million-resume/suite-manifest.json`
- results file: `artifacts/one-million-resume/results.jsonl`
- S3 copy source: `s3://ecaz-cloud-10k-medium-a02e4aea/bench-artifacts/smoke/20260524T231720Z/`

## Setup and Cleanup Artifacts

- `artifacts/s3-final-224632/precheck-host.log`: host, PostgreSQL, memory, and CPU precheck.
- `artifacts/s3-1m-resume-230204/load-1m-diskann-default.log`: failed first 1M resume due `No space left on device`.
- `artifacts/diagnose-db-space/` and `artifacts/diagnose-dataset-space/`: DB and dataset space diagnostics used to identify duplicate retained data.
- `artifacts/drop-partial-1m-single/`: suite-driven drop of the partial 1M table.
- `artifacts/cleanup-duplicate-fetches/`: suite-driven cleanup of duplicate fetched parquet directories.

## Latency

| scale | L64 mean/p50/p95/p99 | L128 mean/p50/p95/p99 | L200 mean/p50/p95/p99 | L400 mean/p50/p95/p99 | L800 mean/p50/p95/p99 |
| --- | --- | --- | --- | --- | --- |
| 10k | 0.99/0.98/1.19/1.40 ms | 1.33/1.32/1.64/2.01 ms | 1.62/1.62/2.10/2.32 ms | 2.37/2.41/3.03/3.42 ms | 3.69/3.64/4.70/4.94 ms |
| 50k | 1.63/1.64/2.10/2.36 ms | 2.34/2.38/3.17/3.42 ms | 2.97/2.97/3.98/4.24 ms | 4.75/4.72/6.28/6.93 ms | 7.94/7.93/9.91/10.8 ms |
| 100k | 1.85/1.81/2.36/2.56 ms | 2.82/2.81/3.44/3.90 ms | 3.83/3.83/4.58/4.82 ms | 6.37/6.47/7.62/8.12 ms | 11.2/11.4/13.3/14.0 ms |
| 1M | 3.90/3.72/5.80/7.69 ms | 5.41/5.20/8.20/9.65 ms | 6.98/6.68/10.8/12.7 ms | 11.4/11.3/18.2/21.3 ms | 19.9/19.7/30.9/35.6 ms |

## Recall

| scale | L64 | L128 | L200 | L400 | L800 |
| --- | --- | --- | --- | --- | --- |
| 10k | 0.9965 | 0.9965 | 0.9970 | 0.9970 | 0.9975 |
| 50k | 0.9585 | 0.9720 | 0.9795 | 0.9845 | 0.9855 |
| 100k | 0.9165 | 0.9625 | 0.9745 | 0.9855 | 0.9865 |
| 1M | 0.9385 | 0.9655 | 0.9735 | 0.9800 | 0.9825 |

## Storage

| scale | rows | total | DiskANN index | index per row |
| --- | ---: | ---: | ---: | ---: |
| 10k | 10,000 | 164.5 MiB | 4.7 MiB | 494.0 B |
| 50k | 50,000 | 821.9 MiB | 23.1 MiB | 484.3 B |
| 100k | 100,000 | 1.6 GiB | 46.1 MiB | 483.1 B |
| 1M | 990,000 | 15.9 GiB | 455.1 MiB | 482.0 B |

## Notes

- The final 1M latency artifact does not include the new `cache_state` column because the run was dispatched before commit `d3a99b2ab`; the checked-in suite configs now label future latency rows as `post_recall_warm`.
- The final suite closes the Task 59 1M proof gap, but the 100k focused-vs-final latency variance is larger than the claimed micro-optimization delta. The supported claim is recall-preserving allocation and scan-loop simplification, not a proven latency improvement.
