# Task 121 Packet 023 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/023-phase3-local-100k-sampled-retune/`
- Head SHA: `f130a3db92b29cdfa0692290328f01479cc88579`
- Suite: `task121-phase3-local-100k-sampled-retune`
- Packet manifest written: `2026-06-25T22:52:13Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Build profile: `release`
- Prefix: `t121_s3_100k_b4_tr50_f8_b64`
- Storage format: `rabitq`
- Rerank mode: default
- Fixture/index source: packet-local 100k load
- Isolation: one-table/one-index 100k surface for this packet
- Status: 6 completed, 0 failed, 0 skipped, 0 stale

## Commands

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-audit.log
```

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-dryrun-manifest.json --log-file reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-dryrun.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-manifest.json --results-output reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-results.jsonl --log-file reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-run.log
```

Status:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/023-phase3-local-100k-sampled-retune/artifacts/suite-phase3-local-100k-sampled-retune-status.log
```

## Status

```text
[suite:task121-phase3-local-100k-sampled-retune] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

The generated `truth-cache-100k-q200-k10.json` file is a local-only cache and is
not committed. The committed evidence is the truth-cache log, recall logs,
storage log, suite manifests, and suite results JSONL.

## Artifacts

- `summary-100k-sampled-retune.md`: compact result summary.
- `suite-phase3-local-100k-sampled-retune.json`: SuiteConfig used for the run.
- `suite-phase3-local-100k-sampled-retune-audit.log`: audit command output.
- `suite-phase3-local-100k-sampled-retune-dryrun.log`: dry-run command output.
- `suite-phase3-local-100k-sampled-retune-dryrun-manifest.json`: dry-run manifest.
- `suite-phase3-local-100k-sampled-retune-run.log`: suite execution log.
- `suite-phase3-local-100k-sampled-retune-manifest.json`: final suite manifest.
- `suite-phase3-local-100k-sampled-retune-results.jsonl`: structured suite results.
- `suite-phase3-local-100k-sampled-retune-status.log`: final status.
- `precheck-host.log`: host/version and pre-run table check.
- `load-100k_b4_tr50_f8_block64.log`: packet-local 100k fixture load.
- `storage-100k_b4_tr50_f8_block64.log`: storage measurement.
- `truth-cache-100k-q200-k10.log`: packet-local truth-cache seed recall log.
- `recall-100k_b4_tr50_f8_block64_off.log`: pruning-off 100k recall.
- `recall-100k_b4_tr50_f8_block64_sampled_loose_g4096_p8192_r4.log`: retuned sampled 100k recall.

## Key Result Lines

Storage:

```text
index=404.8 MiB, index_per_row=4244.8 B
total=1.9 GiB, total_per_row=20915.2 B
```

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=4934.02 ms
```

Recall A/B:

```text
off:     recall@48=0.9945 recall@64=0.9985 recall@96=1.0000
retuned: recall@48=0.9945 recall@64=0.9985 recall@96=1.0000

off mean_q_time:     48=3482.10 ms 64=4090.92 ms 96=4681.82 ms
retuned mean_q_time: 48=3384.60 ms 64=3951.75 ms 96=4253.79 ms
```
