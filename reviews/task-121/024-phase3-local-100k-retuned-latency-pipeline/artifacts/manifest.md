# Task 121 Packet 024 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/`
- Head SHA: `83dd3b107984dc90eb767485f8df2cbc398ec329`
- Suite: `task121-phase3-local-100k-retuned-latency-pipeline`
- Packet manifest written: `2026-06-26T02:30:31Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Build profile: `release`
- Prefix: `t121_s3_100k_b4_tr50_f8_b64`
- Storage format: `rabitq`
- Rerank mode: default
- Fixture/index source: packet 023 100k b4/tr50/f8 block-summary surface
- Isolation: one-table/one-index 100k surface for this packet
- Status: 7 completed, 0 failed, 0 skipped, 0 stale

## Commands

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-audit.log
```

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-dryrun-manifest.json --log-file reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-dryrun.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-manifest.json --results-output reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-results.jsonl --log-file reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-run.log
```

Status:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/artifacts/suite-phase3-local-100k-retuned-latency-pipeline-status.log
```

## Status

```text
[suite:task121-phase3-local-100k-retuned-latency-pipeline] completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

The generated `truth-cache-100k-q200-k10.json` file is a local-only cache and is
not committed. The committed evidence is the truth-cache log, latency logs,
pipeline logs, storage log, suite manifests, and suite results JSONL.

## Artifacts

- `summary-100k-retuned-latency-pipeline.md`: compact result summary.
- `suite-phase3-local-100k-retuned-latency-pipeline.json`: SuiteConfig used for the run.
- `suite-phase3-local-100k-retuned-latency-pipeline-audit.log`: audit command output.
- `suite-phase3-local-100k-retuned-latency-pipeline-dryrun.log`: dry-run command output.
- `suite-phase3-local-100k-retuned-latency-pipeline-dryrun-manifest.json`: dry-run manifest.
- `suite-phase3-local-100k-retuned-latency-pipeline-run.log`: suite execution log.
- `suite-phase3-local-100k-retuned-latency-pipeline-manifest.json`: final suite manifest.
- `suite-phase3-local-100k-retuned-latency-pipeline-results.jsonl`: structured suite results.
- `suite-phase3-local-100k-retuned-latency-pipeline-status.log`: final status.
- `precheck-host.log`: host/version and pre-run table check.
- `storage-100k_b4_tr50_f8_block64.log`: storage measurement.
- `truth-cache-100k-q200-k10.log`: packet-local truth-cache seed recall log.
- `latency-100k_b4_tr50_f8_block64_off.log`: clean latency with pruning off.
- `latency-100k_b4_tr50_f8_block64_retuned_sampled.log`: clean latency with retuned sampled pruning.
- `pipeline-100k_b4_tr50_f8_block64_off.log`: pipeline counters/recall with pruning off.
- `pipeline-100k_b4_tr50_f8_block64_retuned_sampled.log`: pipeline counters/recall with retuned sampled pruning.

## Key Result Lines

Storage:

```text
index=404.8 MiB, index_per_row=4244.8 B
total=1.9 GiB, total_per_row=20915.2 B
```

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=4639.98 ms
```

Clean latency p50:

```text
off:     8=960.5 ms 16=1624.1 ms 32=2678.3 ms 48=3411.2 ms 96=4622.4 ms
retuned: 8=951.4 ms 16=1641.3 ms 32=2685.4 ms 48=3367.9 ms 96=4200.8 ms
```

Pipeline p50 and recall:

```text
off p50:     8=954.048 ms 16=1603.581 ms 32=2618.912 ms 48=3368.828 ms 96=4607.716 ms
retuned p50: 8=946.735 ms 16=1602.378 ms 32=2616.362 ms 48=3372.773 ms 96=4211.934 ms

off recall:     8=0.9330 16=0.9670 32=0.9895 48=0.9945 96=1.0000
retuned recall: 8=0.9330 16=0.9670 32=0.9895 48=0.9945 96=1.0000
```

Pipeline counters:

```text
object bytes unchanged at every nprobe:
8=5504487114 16=11144593728 32=22297157442 48=33073013046 96=64389908578

nprobe=96 candidates: off=76623116 retuned=56982159
nprobe=96 heap_rerank: off=19307246 retuned=17473898
```
