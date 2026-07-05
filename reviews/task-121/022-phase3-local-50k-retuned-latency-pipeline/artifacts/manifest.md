# Task 121 Packet 022 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/`
- Head SHA: `4165414d7df38d701aa8a4ea27204529dab9ead3`
- Suite: `task121-phase3-local-50k-retuned-latency-pipeline`
- Packet manifest written: `2026-06-25T20:30:00Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Build profile: `release`
- Fixture/index source: packet 020 50k b4/tr50/f8 RaBitQ block-summary load
- Prefix: `t121_s3_50k_b4_tr50_f8_b64`
- Storage format: `rabitq`
- Rerank mode: default
- Isolation: reused packet 020 one-table/one-index 50k surface
- Status: 7 completed, 0 failed, 0 skipped, 0 stale

## Commands

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-audit.log
```

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-dryrun-manifest.json --log-file reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-dryrun.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-manifest.json --results-output reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-results.jsonl --log-file reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-run.log
```

Status:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-status.log
```

## Status

```text
[suite:task121-phase3-local-50k-retuned-latency-pipeline] completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

The generated `truth-cache-50k-q200-k10.json` file is a local-only cache and is
not committed. The committed evidence is the truth-cache log, latency logs,
pipeline logs, suite manifests, and suite results JSONL.

## Artifacts

- `summary-50k-retuned-latency-pipeline.md`: compact A/B summary.
- `suite-phase3-local-50k-retuned-latency-pipeline.json`: SuiteConfig used for the run.
- `suite-phase3-local-50k-retuned-latency-pipeline-audit.log`: audit command output.
- `suite-phase3-local-50k-retuned-latency-pipeline-dryrun.log`: dry-run command output.
- `suite-phase3-local-50k-retuned-latency-pipeline-dryrun-manifest.json`: dry-run manifest.
- `suite-phase3-local-50k-retuned-latency-pipeline-run.log`: suite execution log.
- `suite-phase3-local-50k-retuned-latency-pipeline-manifest.json`: final suite manifest.
- `suite-phase3-local-50k-retuned-latency-pipeline-results.jsonl`: structured suite results.
- `suite-phase3-local-50k-retuned-latency-pipeline-status.log`: final status.
- `precheck-host.log`: host/version and table/index precheck.
- `storage-50k_b4_tr50_f8_block64.log`: storage measurement.
- `truth-cache-50k-q200-k10.log`: packet-local truth-cache seed recall log.
- `latency-50k_b4_tr50_f8_block64_off.log`: pruning-off standalone latency.
- `latency-50k_b4_tr50_f8_block64_retuned_sampled.log`: retuned sampled standalone latency.
- `pipeline-50k_b4_tr50_f8_block64_off.log`: pruning-off pipeline counters.
- `pipeline-50k_b4_tr50_f8_block64_retuned_sampled.log`: retuned sampled pipeline counters.

## Key Result Lines

Storage:

```text
index=203.4 MiB, index_per_row=4265.2 B
total=998.4 MiB, total_per_row=20936.9 B
```

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=2029.71 ms
```

Standalone latency:

```text
off:     p50@48=1406.6 ms p50@64=1581.1 ms p50@96=1988.0 ms
retuned: p50@48=1411.5 ms p50@64=1611.8 ms p50@96=1807.6 ms
```

Pipeline:

```text
off:     p50@48=1409.952 ms p50@64=1666.490 ms p50@96=2073.081 ms recall=1.0000/1.0000/1.0000
retuned: p50@48=1384.971 ms p50@64=1548.382 ms p50@96=1737.456 ms recall=1.0000/1.0000/1.0000
off candidates:     48=19,807,598 64=26,111,939 96=37,774,415
retuned candidates: 48=19,807,598 64=26,521,536 96=28,367,005
```

Object-byte counters:

```text
off object_bytes:     48=16,649,378,828 64=21,948,603,032 96=31,752,679,906
retuned object_bytes: 48=16,649,378,828 64=21,948,603,032 96=31,752,679,906
```
