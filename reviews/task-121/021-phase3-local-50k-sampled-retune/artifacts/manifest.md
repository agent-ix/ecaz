# Task 121 Packet 021 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/021-phase3-local-50k-sampled-retune/`
- Head SHA: `371ab43c32848da992e19633f1162438ba56d6c3`
- Suite: `task121-phase3-local-50k-sampled-retune`
- Packet manifest written: `2026-06-25T19:35:00Z`
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
- Status: 3 completed, 0 failed, 0 skipped, 0 stale

## Commands

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-audit.log
```

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-dryrun-manifest.json --log-file reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-dryrun.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-manifest.json --results-output reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-results.jsonl --log-file reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-run.log
```

Status:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-status.log
```

## Status

```text
[suite:task121-phase3-local-50k-sampled-retune] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

The generated `truth-cache-50k-q200-k10.json` file is a local-only cache and is
not committed. The committed evidence is the truth-cache log and the retuned
recall log.

## Artifacts

- `summary-50k-sampled-retune.md`: compact comparison against packet 020.
- `suite-phase3-local-50k-sampled-retune.json`: SuiteConfig used for the run.
- `suite-phase3-local-50k-sampled-retune-audit.log`: audit command output.
- `suite-phase3-local-50k-sampled-retune-dryrun.log`: dry-run command output.
- `suite-phase3-local-50k-sampled-retune-dryrun-manifest.json`: dry-run manifest.
- `suite-phase3-local-50k-sampled-retune-run.log`: suite execution log.
- `suite-phase3-local-50k-sampled-retune-manifest.json`: final suite manifest.
- `suite-phase3-local-50k-sampled-retune-results.jsonl`: structured suite results.
- `suite-phase3-local-50k-sampled-retune-status.log`: final status.
- `precheck-host.log`: host/version and table/index precheck.
- `truth-cache-50k-q200-k10.log`: packet-local truth-cache seed recall log.
- `recall-50k_b4_tr50_f8_block64_sampled_loose_g2048_p4096_r4.log`: retuned recall sweep.

## Key Result Lines

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=2042.74 ms
```

Retuned sampled global pruning:

```text
g2048/p4096/r4: r@48=1.0000 mean@48=1421.46 ms
g2048/p4096/r4: r@64=1.0000 mean@64=1605.34 ms
g2048/p4096/r4: r@96=1.0000 mean@96=1786.64 ms
```
