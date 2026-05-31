# Task 75 Intel Local Routing Envelope Diagnostic Fix Rerun

- head SHA: `f5dea05fcd32e0b871e4b5815371109fb3123883`
- task bucket: `reviews/task-75`
- benchmark packet: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun`
- lane: `intel-local`
- fixture: real-100k DBpedia-style corpus, 200 queries
- storage format: SPIRE `turboquant` for SPIRE rows, IVF `pq_fastscan` control
- rerank mode: heap rerank, effective rerank width 25
- isolated/shared surface: shared-table local surface from the Task 75 suite config
- timestamp: `2026-05-31T13:58:18-07:00`
- suite config: `suite.json`
- suite config SHA256: `350a78f8f5518de226b20deb76052d842d3f86190dd6d35ad28f46ce2139c417`

## Commands

```sh
target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/install-ecaz-pg18.log
```

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18 -l benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/pg18-restart.log
```

```sh
target/debug/ecaz bench suite audit --config benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/suite.json --database task75_spire_gate_fix --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-audit.log
```

```sh
target/debug/ecaz bench suite run --dry-run --config benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/suite.json --database task75_spire_gate_fix --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-dry-run-manifest.json --log-file benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-dry-run.log
```

```sh
target/debug/ecaz bench suite run --config benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/suite.json --database task75_spire_gate_fix --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-manifest.json --log-file benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-run.log
```

```sh
target/debug/ecaz bench suite report --manifest benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-manifest.json --results-output benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/normalized-results.jsonl --log-file benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-report.md
```

## Key Results

| setting | leaf routes | candidate sum | funnel candidate sum | recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| tg16 / nprobe16 | 3,200 | 2,514,557 | 2,514,557 | 0.8525 | 27.536 ms | 32.948 ms |
| tg32 / nprobe32 | 6,400 | 5,165,224 | 5,165,224 | 0.9310 | 50.550 ms | 61.364 ms |
| tg64 / nprobe64 | 12,800 | 10,420,357 | 10,420,357 | 0.9825 | 92.957 ms | 104.996 ms |
| tg96 / nprobe96 | 19,200 | 15,506,227 | 15,506,227 | 0.9975 | 132.203 ms | 148.224 ms |
| tg128 / effective nprobe96 | 19,200 | 15,506,227 | 15,506,227 | 0.9975 | 132.196 ms | 144.428 ms |
| IVF nprobe96 control | n/a | planned 75,000 | n/a | 0.9980 | 36.9 ms | 41.6 ms |

The fixed diagnostic now agrees with the suite runner for every SPIRE candidate sum above.

Per-query diagnostic candidate counts vary after the fix:

| setting | queries | min | max | sum |
| --- | ---: | ---: | ---: | ---: |
| tg16 / nprobe16 | 200 | 9,146 | 15,406 | 2,514,557 |
| tg32 / nprobe32 | 200 | 22,757 | 29,825 | 5,165,224 |
| tg64 / nprobe64 | 200 | 47,310 | 56,751 | 10,420,357 |
| tg96 / nprobe96 | 200 | 72,629 | 81,736 | 15,506,227 |
| tg128 / effective nprobe96 | 200 | 72,629 | 81,736 | 15,506,227 |

## Artifacts

- `artifacts/suite-report.md`: normalized suite report and parsed results
- `artifacts/results.jsonl`: raw suite step results
- `artifacts/normalized-results.jsonl`: report-normalized results
- `artifacts/suite-manifest.json`: suite run manifest
- `artifacts/funnel-100k-tg16-b0.jsonl`: per-query SPIRE funnel diagnostics
- `artifacts/funnel-100k-tg32-b0.jsonl`: per-query SPIRE funnel diagnostics
- `artifacts/funnel-100k-tg64-b0.jsonl`: per-query SPIRE funnel diagnostics
- `artifacts/funnel-100k-tg96-b0.jsonl`: per-query SPIRE funnel diagnostics
- `artifacts/funnel-100k-tg128-b0.jsonl`: per-query SPIRE funnel diagnostics
- `artifacts/suite-audit.log`, `artifacts/suite-dry-run.log`, `artifacts/suite-run.log`: runner logs
- `artifacts/install-ecaz-pg18.log`, `artifacts/pg18-restart.log`: local PG18 extension install and restart logs
