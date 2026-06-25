# Task 121 review request: Phase 3 local 50k retuned pruning latency/pipeline

## Scope

Packet 021 found a 50k sampled global pruning retune that recovered recall
against the packet 020 aggressive/lossy policy:

```text
max_global_blocks=2048
global_probe_blocks=4096
sample_rows_per_block=4
sample_summary_prior_weight=0.8
summary_radius_weight=0.25
route_prior_weight=0.0
```

This packet carries that retune forward into the owed 50k latency and SPIRE
pipeline A/B on the same packet 020 50k b4/tr50/f8 RaBitQ block-summary index.
It covers nprobe 48, 64, and 96, the saturated recall checkpoints from packet
021. This packet does not close Task 121.

## Validation

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-audit.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-manifest.json --results-output reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-results.jsonl --log-file reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/artifacts/suite-phase3-local-50k-retuned-latency-pipeline-run.log
```

Status:

```text
completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

Storage for the reused 50k index:

| index | index/row | total | total/row |
|---:|---:|---:|---:|
| 203.4 MiB | 4265.2 B | 998.4 MiB | 20936.9 B |

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=2029.71 ms
```

Standalone latency:

| nprobe | off p50 | retuned sampled p50 | p50 delta | off p95 | retuned sampled p95 | p95 delta |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 1406.6 ms | 1411.5 ms | +4.9 ms | 1691.4 ms | 1643.9 ms | -47.5 ms |
| 64 | 1581.1 ms | 1611.8 ms | +30.7 ms | 1925.1 ms | 1908.2 ms | -16.9 ms |
| 96 | 1988.0 ms | 1807.6 ms | -180.4 ms | 2340.2 ms | 2010.6 ms | -329.6 ms |

Pipeline:

| nprobe | off recall@10 | retuned recall@10 | off p50 | retuned p50 | p50 delta | off candidates | retuned candidates |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 48 | 1.0000 | 1.0000 | 1409.952 ms | 1384.971 ms | -24.981 ms | 19,807,598 | 19,807,598 |
| 64 | 1.0000 | 1.0000 | 1666.490 ms | 1548.382 ms | -118.108 ms | 26,111,939 | 26,521,536 |
| 96 | 1.0000 | 1.0000 | 2073.081 ms | 1737.456 ms | -335.625 ms | 37,774,415 | 28,367,005 |

Object-byte counters were unchanged for this local pipeline surface:

| nprobe | off object bytes | retuned object bytes |
|---:|---:|---:|
| 48 | 16,649,378,828 | 16,649,378,828 |
| 64 | 21,948,603,032 | 21,948,603,032 |
| 96 | 31,752,679,906 | 31,752,679,906 |

## Recommendation

Carry the 50k retuned policy forward only as a high-nprobe recall-neutral
candidate. It is not a broad low/mid-nprobe win: standalone latency is flat at
48 and worse at 64, but nprobe 96 improves materially in both standalone
latency and pipeline while preserving recall 1.0000.

The local pipeline object-byte counters do not show read-byte reduction. The
main positive signal is 50k nprobe 96: pipeline p50 improves by 335.625 ms and
candidate count drops by 9,407,410 over 200 queries.

Still owed for Task 121 Phase 3:

- 100k recall retune plus latency/storage/pipeline
- 10k/50k/100k final scan-efficiency A/B summary
- default/TurboQuant block-summary coverage or explicit implementation-gap
  decision
- final Pareto/verdict tying the surviving lever to route-stage loss

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-50k-retuned-latency-pipeline.md`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline.json`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-audit.log`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-dryrun.log`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-dryrun-manifest.json`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-run.log`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-manifest.json`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-results.jsonl`
- `artifacts/suite-phase3-local-50k-retuned-latency-pipeline-status.log`
- `artifacts/precheck-host.log`
- `artifacts/storage-50k_b4_tr50_f8_block64.log`
- `artifacts/truth-cache-50k-q200-k10.log`
- `artifacts/latency-50k_b4_tr50_f8_block64_off.log`
- `artifacts/latency-50k_b4_tr50_f8_block64_retuned_sampled.log`
- `artifacts/pipeline-50k_b4_tr50_f8_block64_off.log`
- `artifacts/pipeline-50k_b4_tr50_f8_block64_retuned_sampled.log`
