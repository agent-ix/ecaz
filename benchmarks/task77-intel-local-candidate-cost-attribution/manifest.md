# Task 77 Intel-Local Candidate Cost Attribution

- head SHA measured: `51293e7531bc1bc29393bff22ed75c909c12e474`
- task bucket: `reviews/task-77/`
- benchmark packet: `benchmarks/task77-intel-local-candidate-cost-attribution/`
- host lane: `intel-local`
- timestamp: `2026-05-31T23:04:30Z`
- runner: `ecaz bench suite`
- suite config: `suite.json`
- suite manifest: `artifacts/suite-manifest.json`
- status: `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- fixture: `ec_real_100k`, `queries_limit=200`, `boundary_replica_count=0`
- storage format: `turboquant`
- rerank mode: `rerank_width=25`
- isolated/shared surface: isolated one-index-per-table Task 77 prefix

## Commands

```bash
cargo build -p ecaz-cli --no-default-features
target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/install-ecaz-pg18.log
script -q -c '/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18 -m fast -l /home/peter/.pgrx/data-18/task77-pg18-server.log' benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/pg18-restart-command.log
target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-audit.log bench suite audit --config benchmarks/task77-intel-local-candidate-cost-attribution/suite.json
target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-dry-run.log --database task77_spire_attribution --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config benchmarks/task77-intel-local-candidate-cost-attribution/suite.json --manifest-output benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-dry-run-manifest.json
target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-run.log --database task77_spire_attribution --host /home/peter/.pgrx --port 28818 bench suite run --config benchmarks/task77-intel-local-candidate-cost-attribution/suite.json --manifest-output benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json --results-output benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/results.jsonl
target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-status.log bench suite status --manifest benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-report.log bench suite report --manifest benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json --results-output benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/report-results.jsonl
```

## Key Results

| point | candidates | mean candidates/query | p50 query | recall@10 | p50 object read | p50 score | p50 materialize | p50 heap append | score share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tg64/nprobe64 | 10,420,357 | 52,101 | 92.017 ms | 0.9825 | 10.984 ms | 69.163 ms | 1.743 ms | 1.257 ms | 82.9% |
| tg96/nprobe96 | 15,506,227 | 77,531 | 133.226 ms | 0.9975 | 17.934 ms | 102.464 ms | 2.588 ms | 1.870 ms | 82.1% |
| tg128/nprobe128 | 20,000,000 | 100,000 | 169.329 ms | 1.0000 | 20.806 ms | 132.107 ms | 3.381 ms | 2.396 ms | 83.2% |

Final rerank/top-k handoff count was stable across all points: `5,000`
retained candidates and `2,000` returned rows over 200 queries. Production
read profile reported local-heap-candidate total p50 of `87 ms`, `129 ms`, and
`165 ms` for nprobe 64, 96, and 128 respectively.

The fixed-candidate microbench used `77,531` candidates, `1536` dimensions,
`256` iterations, and `avx2+fma`. Relevant batch rows:

| variant | scores | ns/score |
| --- | ---: | ---: |
| bits1 batch | 19,847,936 | 7,168.09 |
| bits4 batch | 19,847,936 | 10,774.52 |
| bits8 batch | 19,847,936 | 9,066.04 |
| bits8c3 batch | 19,847,936 | 9,100.28 |
| bits8c4 batch | 19,847,936 | 9,047.91 |

## Artifacts

- `artifacts/install-ecaz-pg18.log`
- `artifacts/pg18-restart-command.log`
- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-dry-run-manifest.json`
- `artifacts/suite-run.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/funnel-100k-tg64-b0.jsonl`
- `artifacts/funnel-100k-tg96-b0.jsonl`
- `artifacts/funnel-100k-tg128-b0.jsonl`
- `artifacts/funnel-attribution-summary.json`
- `artifacts/fixed-candidate-rabitq-kernel-77531.log`
