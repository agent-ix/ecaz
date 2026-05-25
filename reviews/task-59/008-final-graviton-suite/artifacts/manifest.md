# Task 59 Packet 008 Artifact Manifest

- head SHA: `130b70592032cd91ab7c204ac14ee63a88fdda5c`
- task bucket: `reviews/task-59/008-final-graviton-suite`
- packet path: `reviews/task-59/008-final-graviton-suite`
- benchmark packet: `benchmarks/task59-aws-diskann-final-graviton-suite`
- timestamp: `2026-05-25T02:47:23Z`
- lane: AWS Graviton DiskANN final suite
- fixture: `ec_real_10k`, `ec_real_50k`, `ec_real_100k`, retained `ec_real_ann_benchmarks_anchor` 1M staging
- storage format: `pq_fastscan`
- rerank mode: default DiskANN scan path, `rerank_budget=64`
- isolated one-index-per-table: yes

## Durable Benchmark Artifacts

- `benchmarks/task59-aws-diskann-final-graviton-suite/manifest.md`
- `benchmarks/task59-aws-diskann-final-graviton-suite/results-summary.md`
- `benchmarks/task59-aws-diskann-final-graviton-suite/suite.json`
- `benchmarks/task59-aws-diskann-final-graviton-suite/suite-1m-resume.json`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/s3-final-224632/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/one-million-resume/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/s3-1m-resume-230204/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/diagnose-db-space/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/diagnose-dataset-space/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/drop-partial-1m-single/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/cleanup-duplicate-fetches/`

## Key Result Lines

- 1M load/index total: `12357.42s`
- 1M DiskANN build: `11682.44s`
- 1M latency L64/L128/L200/L400/L800 means: `3.90`, `5.41`, `6.98`, `11.4`, `19.9` ms
- 1M recall L64/L128/L200/L400/L800: `0.9385`, `0.9655`, `0.9735`, `0.9800`, `0.9825`
- 1M storage: `15.9 GiB` total, `455.1 MiB` DiskANN index, `482.0 B` index bytes/row
- split-suite reason: first final suite run succeeded through 100k and failed 1M on a chunked-manifest mismatch; the resume suite used retained single-TSV 1M staging and completed.
