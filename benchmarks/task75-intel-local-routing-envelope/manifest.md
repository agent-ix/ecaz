# Task 75 Intel Local Routing Envelope

- Task: `plan/tasks/75-spire-latency-routing-envelope.md`
- Branch: `task-75-spire-latency-routing-envelope`
- Run head SHA: `4f6de38964403a415a9a5b26cd0d71ec305914bb`
- Packet: `benchmarks/task75-intel-local-routing-envelope`
- Host lane: `intel-local`
- Timestamp: `2026-05-31T18:55:38Z`
- PostgreSQL: PG18.3 via `/home/peter/.pgrx`, socket dir `/home/peter/.pgrx`, port `28818`
- Database: `task75_spire_gate`
- Suite config: `suite.json`
- Suite config SHA256: `dc0666aa7348bec5fa4f52cc35d6f66a49bff848785297a2e7f100ddceea6b40`
- Suite artifacts: `artifacts/suite-manifest.json`, `artifacts/results.jsonl`, `artifacts/normalized-results.jsonl`, `artifacts/suite-report.md`
- Surface isolation: one index per logical surface. SPIRE tg16/tg32/tg64/tg96/tg128 points reuse the same table prefix with explicit index rebuilds; IVF uses a separate `task75_ivf_100k_control` prefix.
- AWS state during local work: `artifacts/aws-status-1m-after-local-run.log` shows profile `1m` paused with DB instance stopped and `$0/hr`; `artifacts/aws-status-10k-medium-after-local-run.log` shows profile `10k-medium` down and `$0/hr`.

## Commands

```bash
target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/install-ecaz-pg18.log
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18 -l benchmarks/task75-intel-local-routing-envelope/artifacts/pg18-restart.log
target/debug/ecaz bench suite audit --config benchmarks/task75-intel-local-routing-envelope/suite.json --database task75_spire_gate --host /home/peter/.pgrx --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/suite-audit.log
target/debug/ecaz bench suite run --dry-run --config benchmarks/task75-intel-local-routing-envelope/suite.json --database task75_spire_gate --host /home/peter/.pgrx --manifest-output benchmarks/task75-intel-local-routing-envelope/artifacts/suite-dry-run-manifest.json --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/suite-dry-run.log
target/debug/ecaz bench suite run --config benchmarks/task75-intel-local-routing-envelope/suite.json --database task75_spire_gate --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task75-intel-local-routing-envelope/artifacts/suite-manifest.json --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/suite-run-rerun-port28818.log
target/debug/ecaz bench suite report --manifest benchmarks/task75-intel-local-routing-envelope/artifacts/suite-manifest.json --results-output benchmarks/task75-intel-local-routing-envelope/artifacts/normalized-results.jsonl --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/suite-report.md
```

`artifacts/suite-run.log` records the first failed run attempt. It failed before measurement because the suite connection omitted `--port 28818`; `artifacts/suite-run-rerun-port28818.log` is the successful run.

## Key Results

SPIRE 100k, `storage_format=turboquant`, `boundary_replica_count=0`, `rerank_width=25`, 200 query sample:

| Point | nprobe | recall@10 | p50 | p95 | leaf routes | candidates | retained | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tg16 b0 | 16 | 0.8525 | 26.814 ms | 33.414 ms | 2,666 | 2,087,914 | 5,000 | 2,000 |
| tg32 b0 | 32 | 0.9310 | 48.199 ms | 54.407 ms | 3,533 | 2,769,013 | 5,000 | 2,000 |
| tg64 b0 | 64 | 0.9825 | 90.643 ms | 100.316 ms | 3,556 | 2,784,952 | 5,000 | 2,000 |
| tg96 b0 | 96 | 0.9975 | 131.292 ms | 143.238 ms | 3,556 | 2,784,952 | 5,000 | 2,000 |
| tg128 b0 | 96 | 0.9975 | 134.271 ms | 145.134 ms | 3,556 | 2,784,952 | 5,000 | 2,000 |

IVF 100k control, `storage_format=pq_fastscan`, `nlists=128`, `nprobe=96`, `rerank_width=500`, 200 query sample:

| Point | recall@10 | mean q-time | p50 | p95 | estimated candidates | observed postings visited | rerank rows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| IVF nprobe96 | 0.9980 | 37.85 ms | 37.0 ms | 42.0 ms | 75,000 | 77,760 | 500 |

The high-recall SPIRE local point matches IVF recall within this sample (`0.9975` vs `0.9980`) but is materially slower on this Intel desktop (`131-134 ms` p50 vs `37 ms` p50). SPIRE candidate volume saturates by tg64/tg96 at about 2.78M leaf candidates across 200 queries while only 5,000 rows reach heap rerank.

## Artifact Index

- Setup: `artifacts/install-ecaz-pg18.log`, `artifacts/pg18-restart.log`, `artifacts/drop-db.log`, `artifacts/create-db.log`
- Audit and runner metadata: `artifacts/suite-audit.log`, `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`, `artifacts/suite-manifest.json`, `artifacts/suite-run-rerun-port28818.log`, `artifacts/results.jsonl`, `artifacts/normalized-results.jsonl`, `artifacts/suite-report.md`
- SPIRE logs: `artifacts/load-100k-spire-tg16-b0.log`, `artifacts/rebuild-100k-tg32-b0.log`, `artifacts/rebuild-100k-tg64-b0.log`, `artifacts/rebuild-100k-tg96-b0.log`, `artifacts/rebuild-100k-tg128-b0.log`, `artifacts/pipeline-100k-tg16-b0.log`, `artifacts/pipeline-100k-tg32-b0.log`, `artifacts/pipeline-100k-tg64-b0.log`, `artifacts/pipeline-100k-tg96-b0.log`, `artifacts/pipeline-100k-tg128-b0.log`
- SPIRE funnel JSONL: `artifacts/funnel-100k-tg16-b0.jsonl`, `artifacts/funnel-100k-tg32-b0.jsonl`, `artifacts/funnel-100k-tg64-b0.jsonl`, `artifacts/funnel-100k-tg96-b0.jsonl`, `artifacts/funnel-100k-tg128-b0.jsonl`
- IVF control: `artifacts/load-100k-ivf-control.log`, `artifacts/recall-100k-ivf-control-nprobe96.log`, `artifacts/latency-100k-ivf-control-nprobe96.log`, `artifacts/explain-100k-ivf-control-nprobe96.sql`, `artifacts/explain-100k-ivf-control-nprobe96.log`
- AWS status: `artifacts/aws-status-1m-after-local-run.log`, `artifacts/aws-status-10k-medium-after-local-run.log`
