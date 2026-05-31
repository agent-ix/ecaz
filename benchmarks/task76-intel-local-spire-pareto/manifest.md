# Task 76 Intel-Local SPIRE Pareto Manifest

- head SHA: `4d832cdd4533e59864311e6e8918ce43ef63fddf`
- task bucket: `reviews/task-76/001-pareto-measurement/`
- benchmark packet: `benchmarks/task76-intel-local-spire-pareto/`
- host class: `intel-local`
- timestamp: `2026-05-31T19:53:33Z`
- suite config: `benchmarks/task76-intel-local-spire-pareto/suite.json`
- suite config SHA256: `77ed5b5faf914eb7484990e5960cd5d95a74c3e96ff1ca3f232f196167eba0e6`
- database: `task76_spire_pareto`
- socket / port: `/home/peter/.pgrx`, `28818`
- PG target: PG18
- run surface: local Intel desktop, single-node local scans, isolated one-index-per-table task prefixes
- fixture scope: 10k and 100k real corpus fixtures; 1M canonical TSV fixture was not available locally

## Commands

```bash
target/debug/ecaz bench suite audit --config benchmarks/task76-intel-local-spire-pareto/suite.json --database task76_spire_pareto --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-audit.log
target/debug/ecaz bench suite run --dry-run --config benchmarks/task76-intel-local-spire-pareto/suite.json --database task76_spire_pareto --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task76-intel-local-spire-pareto/artifacts/suite-dry-run-manifest.json --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-dry-run.log
target/debug/ecaz bench suite run --config benchmarks/task76-intel-local-spire-pareto/suite.json --database task76_spire_pareto --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task76-intel-local-spire-pareto/artifacts/suite-manifest.json --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-run.log
target/debug/ecaz bench suite report --manifest benchmarks/task76-intel-local-spire-pareto/artifacts/suite-manifest.json --results-output benchmarks/task76-intel-local-spire-pareto/artifacts/normalized-results.jsonl --log-file benchmarks/task76-intel-local-spire-pareto/artifacts/suite-report.md
```

## Artifacts

- `artifacts/suite-audit.log`: suite audit result, 33 steps.
- `artifacts/suite-dry-run.log`: dry-run validation.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest.
- `artifacts/suite-run.log`: full run console log.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: raw suite results.
- `artifacts/normalized-results.jsonl`: parsed report output.
- `artifacts/suite-report.md`: suite report; 33 completed, 0 failed, 0 skipped, 0 missing artifacts.
- `artifacts/load-*.log`, `artifacts/rebuild-*.log`, `artifacts/pipeline-*.log`, `artifacts/recall-*.log`, `artifacts/latency-*.log`: packet-local step logs.

## Key Results

10k SPIRE reached near-perfect recall cheaply:

| Step | nprobe | recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| `pipeline-10k-tg16-b0` | 8 | 0.9975 | 7.956 ms | 8.393 ms |
| `pipeline-10k-tg16-b0` | 16 | 0.9995 | 11.669 ms | 12.535 ms |
| `pipeline-10k-tg32-b0` | 32 | 1.0000 | 19.515 ms | 25.919 ms |
| `pipeline-10k-tg128-b0` | 128 | 1.0000 | 19.712 ms | 20.608 ms |

100k SPIRE recall improved only by paying much higher latency:

| Step | nprobe | recall@10 | p50 | p95 | leaf routes | candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `pipeline-100k-tg16-b0` | 8 | 0.7250 | 15.820 ms | 18.451 ms | 1,546 | 1,193,935 |
| `pipeline-100k-tg16-b0` | 16 | 0.8525 | 26.373 ms | 30.224 ms | 2,666 | 2,087,914 |
| `pipeline-100k-tg32-b0` | 32 | 0.9310 | 48.362 ms | 54.251 ms | 3,533 | 2,769,013 |
| `pipeline-100k-tg64-b0` | 64 | 0.9825 | 98.584 ms | 112.208 ms | 3,556 | 2,784,952 |
| `pipeline-100k-tg96-b0` | 96 | 0.9975 | 146.693 ms | 175.128 ms | 3,556 | 2,784,952 |
| `pipeline-100k-tg128-b0` | 128 | 1.0000 | 172.401 ms | 205.287 ms | 3,556 | 2,784,952 |

100k IVF control dominated the high-recall local latency envelope:

| Control | setting | recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| IVF 100k | nprobe 48 | 0.9805 | 27.6 ms | 30.1 ms |
| IVF 100k | nprobe 80 | 0.9950 | 35.0 ms | 43.0 ms |
| IVF 100k | nprobe 96 | 0.9980 | 37.7 ms | 46.5 ms |
| IVF 100k | nprobe 128 | 1.0000 | 43.1 ms | 50.1 ms |

100k HNSW control was fast but below the target recall range for this task:

| Control | setting | recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| HNSW 100k | ef_search 64 | 0.8295 | 5.06 ms | 9.42 ms |
| HNSW 100k | ef_search 128 | 0.9020 | 7.39 ms | 12.1 ms |
| HNSW 100k | ef_search 200 | 0.9245 | 8.96 ms | 14.6 ms |
| HNSW 100k | ef_search 400 | 0.9385 | 15.6 ms | 22.1 ms |

## Decision

Do not change SPIRE defaults from this local Intel packet.

The 10k fixture has cheap high-recall SPIRE points, but 100k remains the governing case. On 100k, SPIRE needs nprobe 96 to reach 0.9975 recall, with p50 146.693 ms and p95 175.128 ms, while IVF nprobe 96 reaches 0.9980 recall with p50 37.7 ms and p95 46.5 ms. Top-graph search-list width does not reduce the 100k candidate plateau after nprobe 64; leaf routes and candidates stay flat at 3,556 / 2,784,952, while latency keeps increasing with higher nprobe.

The 1M fixture gap prevents promoting a 1M-informed default. The next useful SPIRE work should target the candidate/materialization cost directly rather than raising recall defaults.
