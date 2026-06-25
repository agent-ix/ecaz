# Task 121 Packet 018 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/018-phase2-local-100k-b8-latency-followup/`
- Head SHA: `dfe408f31672aa3830bc7f9a6d66c18810f4dd63`
- Suite: `task121-phase2-local-100k-b8-latency-followup`
- Packet manifest written: `2026-06-25T17:20:01Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Build profile: `release`
- Fixture: `data/staged-current/ec_real_100k_corpus.tsv`
- Fixture SHA from loader logs: `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- Query SHA from loader logs: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage format: `rabitq`
- Rerank mode: default
- Isolation: one table/index per cell; no shared-table surface

## Command

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite run --config reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup.json --manifest-output reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup-manifest.json --results-output reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup-results.jsonl --log-file reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup.log
```

Audit:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite audit --config reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup.json --log-file reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup-audit.log
```

## Status

```text
steps=12
succeeded=12
results_jsonl_lines=263
```

Pipeline diagnostic row counts:

```text
pipeline-100k_b8_tr50_f8-funnel.jsonl=1800
pipeline-100k_b8_tr50_f8-stage-containment.jsonl=10800
pipeline-100k_b8_tr50_f16-funnel.jsonl=1800
pipeline-100k_b8_tr50_f16-stage-containment.jsonl=10800
```

The streamed `pipeline-100k_b8_tr50_f*-funnel.jsonl`,
`pipeline-100k_b8_tr50_f*-stage-containment.jsonl`, and
`truth-cache-100k-q200-k10.json` files are local-only diagnostics and are not
commit artifacts.

## Artifacts

- `summary-100k-b8-latency-followup.md`: compact storage, recall, and latency summary.
- `suite-phase2-local-100k-b8-latency-followup.json`: SuiteConfig used for the run.
- `suite-phase2-local-100k-b8-latency-followup-audit.log`: audit command output.
- `suite-phase2-local-100k-b8-latency-followup.log`: suite execution log.
- `suite-phase2-local-100k-b8-latency-followup-manifest.json`: final suite manifest.
- `suite-phase2-local-100k-b8-latency-followup-results.jsonl`: structured suite result rows.
- `precheck-host.log`: host precheck.
- `load-100k_b8_tr50_f*.log`: b8 load/build logs.
- `storage-100k_b8_tr50_f*.log`: b8 storage logs.
- `pipeline-100k_b8_tr50_f*.log`: b8 compact pipeline logs.
- `latency-100k_b*_tr50_f*.log`: clean cache-warm latency logs for b4 and b8 finalist cells.
- `truth-cache-100k-q200-k10.log`: truth-cache build log.

## Key Result Lines

Pipeline recall:

```text
b8_tr50_f8:  r@8=0.9630 r@16=0.9830 r@32=0.9970 r@48=0.9985 r@64=1.0000 r@96=1.0000 p50@32=3477.841 ms p50@96=5032.966 ms
b8_tr50_f16: r@8=0.9680 r@16=0.9900 r@32=0.9980 r@48=0.9990 r@64=1.0000 r@96=1.0000 p50@32=3528.944 ms p50@96=5058.282 ms
```

Clean cache-warm latency:

```text
b4_tr50_f8:  p50@8=955.0 ms  p50@16=1629.8 ms p50@32=2699.4 ms p50@48=3451.9 ms p50@96=4730.6 ms
b4_tr50_f16: p50@8=984.3 ms  p50@16=1660.4 ms p50@32=2732.8 ms p50@48=3497.3 ms p50@96=4708.2 ms
b8_tr50_f8:  p50@8=1681.7 ms p50@16=2661.6 ms p50@32=3595.2 ms p50@48=4250.1 ms p50@96=5215.1 ms
b8_tr50_f16: p50@8=1673.1 ms p50@16=2565.5 ms p50@32=3624.8 ms p50@48=4294.1 ms p50@96=5283.8 ms
```

Storage:

```text
b8_tr50_f8:  index=704.7 MiB index_bytes_per_row=7389.3 B total_table=2.2 GiB
b8_tr50_f16: index=704.8 MiB index_bytes_per_row=7390.0 B total_table=2.2 GiB
```
