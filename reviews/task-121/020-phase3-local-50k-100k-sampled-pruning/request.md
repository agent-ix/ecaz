# Task 121 review request: Phase 3 local 50k sampled pruning recall check

## Scope

This packet extends packet 019's 10k sampled-pruning pilot to 50k for the
b4/tr50/f8 RaBitQ block-summary candidate. It reuses the packet 019 SuiteConfig
with `--only-tag 50k --only-tag 100k` and an artifact-dir override into this
packet.

Completed evidence:

- 50k load/build
- 50k storage
- 50k truth-cache seed recall
- 50k pruning-off recall sweep
- 50k sampled-pruning recall sweep

The suite was interrupted after the 50k sampled recall sweep, before latency,
pipeline, and all 100k steps. The status snapshot reports 5 completed, 0 failed,
10 skipped, and 13 pending/stale steps.

This is not Task 121 closeout evidence.

## Validation

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --only-tag 50k --only-tag 100k --artifact-dir reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-dryrun-manifest.json --log-file reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-dryrun.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --only-tag 50k --only-tag 100k --artifact-dir reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-manifest.json --results-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-results.jsonl --log-file reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-run.log
```

Status after interrupt:

```text
completed=5 failed=0 skipped=10 dry_run=0 missing_artifacts=0 stale=13
```

## Result

The 50k storage footprint for the block-summary RaBitQ index is:

```text
index=203.4 MiB
index_per_row=4265.2 B
total=998.3 MiB
total_per_row=20936.6 B
```

The recall A/B shows that the sampled setting is not recall-neutral at 50k:

| nprobe | off recall@10 | sampled recall@10 | off mean q-time | sampled mean q-time |
|---:|---:|---:|---:|---:|
| 8 | 0.9810 | 0.9810 | 371.82 ms | 379.34 ms |
| 16 | 0.9905 | 0.9905 | 603.99 ms | 614.80 ms |
| 32 | 0.9985 | 0.9985 | 1062.88 ms | 1040.89 ms |
| 48 | 1.0000 | 0.9995 | 1423.23 ms | 1115.16 ms |
| 64 | 1.0000 | 0.9995 | 1625.24 ms | 1200.44 ms |
| 96 | 1.0000 | 0.9995 | 2045.17 ms | 1261.73 ms |

This is a useful negative/retuning signal. Sampled global pruning materially
reduces high-nprobe runtime, but this exact 50k policy gives back one recall
trial out of 2000 at the saturated checkpoints where the off policy is perfect.

## Recommendation

Do not carry the current 50k sampled setting directly into latency/pipeline/100k
closeout. Retune first with a less aggressive sampled policy, then run narrower
Phase 3 follow-up suites at the recall-saturated checkpoints. The obvious next
settings are larger `max_global_blocks` / `global_probe_blocks` and/or more
`sample_rows_per_block`.

Still owed for Phase 3:

- retuned 50k recall plus latency/pipeline
- 100k recall, latency, storage, and pipeline
- default/TurboQuant block-summary coverage or an explicit implementation gap
  decision

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-50k-recall-sampled-pruning.md`
- `artifacts/suite-phase3-local-50k-100k-sampled-pruning-dryrun.log`
- `artifacts/suite-phase3-local-50k-100k-sampled-pruning-dryrun-manifest.json`
- `artifacts/suite-phase3-local-50k-100k-sampled-pruning-run.log`
- `artifacts/suite-phase3-local-50k-100k-sampled-pruning-manifest.json`
- `artifacts/suite-phase3-local-50k-100k-sampled-pruning-status.log`
- `artifacts/precheck-host.log`
- `artifacts/load-50k_b4_tr50_f8_block64.log`
- `artifacts/storage-50k_b4_tr50_f8_block64.log`
- `artifacts/truth-cache-50k-q200-k10.log`
- `artifacts/recall-50k_b4_tr50_f8_block64_off.log`
- `artifacts/recall-50k_b4_tr50_f8_block64_sampled.log`
