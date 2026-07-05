# Task 121 review request: Phase 3 local 10k RaBitQ sampled pruning pilot

## Scope

This packet starts the Phase 3 scan-efficiency A/B with a local 10k pilot for
the b4/tr50/f8 RaBitQ candidate using block summaries:

- `storage_format=rabitq`
- `leaf_block_rows=64`
- `leaf_block_summary_representatives=2`
- pruning off versus sampled global pruning
- sampled setting: `max_global_blocks=384`, `global_probe_blocks=768`,
  `sample_rows_per_block=4`, `sample_summary_prior_weight=0.8`,
  `summary_radius_weight=0.25`, `route_prior_weight=0.0`

The suite config includes the intended 10k/50k/100k matrix, but this run was
intentionally interrupted after the completed 10k slice to avoid spending the
whole turn on the larger scales before checking the signal. The suite status
snapshot reports 10 completed, 0 failed, and 18 pending/stale steps.

This is not Task 121 closeout evidence. It is a first Phase 3 slice.

## Validation

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-audit.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-manifest.json --results-output reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-results.jsonl --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-run.log
```

Status after interrupt:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-status.log
```

Status:

```text
completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=18
```

## Result

Sampled global pruning preserved 10k recall exactly across the sweep:

| nprobe | off recall@10 | sampled recall@10 |
|---:|---:|---:|
| 8 | 0.9945 | 0.9945 |
| 16 | 0.9980 | 0.9980 |
| 32 | 0.9985 | 0.9985 |
| 48 | 0.9995 | 0.9995 |
| 64 | 1.0000 | 1.0000 |
| 96 | 1.0000 | 1.0000 |

Latency is neutral to worse at low nprobe but improves at the high recall-
saturated points:

| nprobe | off p50 | sampled p50 |
|---:|---:|---:|
| 8 | 68.4 ms | 72.8 ms |
| 16 | 102.7 ms | 102.5 ms |
| 32 | 144.0 ms | 146.6 ms |
| 48 | 202.9 ms | 210.7 ms |
| 64 | 254.0 ms | 248.6 ms |
| 96 | 340.5 ms | 285.6 ms |

Pipeline counters explain the high-nprobe win. At nprobe 96, candidates and
heap-rerank rows dropped from 7,463,419 to 5,121,349 with recall still at
1.0000 and pipeline p50 dropping from 342.133 ms to 283.740 ms. Object bytes
did not drop in this local scan path.

The storage cost of this 10k block-summary RaBitQ index is high: 42.1 MiB
index size, 4415.5 B/index-row, 201.2 MiB total table footprint, and
21094.4 B/row total footprint.

## Recommendation

Carry sampled global block pruning forward to the required 50k and 100k A/B
matrix because the 10k slice shows recall-neutral high-nprobe scan-efficiency
improvement. Do not treat this as sufficient to close Phase 3: 50k/100k recall,
latency, storage, and pipeline counters are still required.

Also keep the default/TurboQuant block-summary item open. The current pruning
surface used here is RaBitQ-only, so this packet does not satisfy that part of
the Phase 3 lever list.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-10k-sampled-pruning.md`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning.json`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning-audit.log`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning-dryrun.log`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning-dryrun-manifest.json`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning-run.log`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning-manifest.json`
- `artifacts/suite-phase3-local-rabitq-sampled-pruning-status.log`
- `artifacts/precheck-host.log`
- `artifacts/pg18-restart-before-phase3.log`
- `artifacts/pg18-ready-before-phase3.log`
- `artifacts/load-10k_b4_tr50_f8_block64.log`
- `artifacts/storage-10k_b4_tr50_f8_block64.log`
- `artifacts/truth-cache-10k-q200-k10.log`
- `artifacts/recall-10k_b4_tr50_f8_block64_off.log`
- `artifacts/recall-10k_b4_tr50_f8_block64_sampled.log`
- `artifacts/latency-10k_b4_tr50_f8_block64_off.log`
- `artifacts/latency-10k_b4_tr50_f8_block64_sampled.log`
- `artifacts/pipeline-10k_b4_tr50_f8_block64_off.log`
- `artifacts/pipeline-10k_b4_tr50_f8_block64_sampled.log`
