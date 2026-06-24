# Task 121 review request: Phase 2 local 50k b2 checkpoint

## Scope

This packet responds to the reviewer gap that 50k b2/b4 f8 cells were missing.
It is an interim checkpoint at the user-requested halt boundary:

- completed: b2/tr10 and b2/tr50 recall/storage at 50k f8
- completed: b4/tr10 and b4/tr50 storage at 50k f8
- pending: b4/tr10 and b4/tr50 recall pipelines

This is local-only evidence on the existing PG18 benchmark database. It is not
AWS evidence. It is also not the Phase 0 local multi-node lane; all measured
tables are in one local PostgreSQL instance and this packet should not be used
as multi-node closeout evidence.

## Validation

Audit:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json
```

Initial run:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.log
```

Resume:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --resume-from reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-resume.log
```

Status:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite status --manifest reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-status.log
```

The current manifest status is `completed=12 failed=0 skipped=0 dry_run=0
missing_artifacts=0 stale=2`. The stale/pending steps are the b4 pipeline cells,
which were halted at the requested breakpoint after b2/tr50 completed.

## Result

Storage for the full b2/b4 f8 50k slice:

| cell | index | per-row index | table total |
|---|---:|---:|---:|
| b2_tr10_f8 | 118.7 MiB | 2488.7 B | 913.6 MiB |
| b2_tr50_f8 | 118.8 MiB | 2490.4 B | 913.7 MiB |
| b4_tr10_f8 | 196.9 MiB | 4129.6 B | 991.9 MiB |
| b4_tr50_f8 | 196.9 MiB | 4128.8 B | 991.8 MiB |

Completed b2 recall/latency pipeline rows:

| cell | nprobe | p50 | p95 | recall@10 |
|---|---:|---:|---:|---:|
| b2_tr10_f8 | 4 | 157.582 ms | 208.783 ms | 0.9205 |
| b2_tr10_f8 | 8 | 257.195 ms | 345.027 ms | 0.9525 |
| b2_tr10_f8 | 12 | 350.241 ms | 461.687 ms | 0.9690 |
| b2_tr10_f8 | 16 | 439.302 ms | 576.116 ms | 0.9730 |
| b2_tr10_f8 | 24 | 630.501 ms | 764.504 ms | 0.9865 |
| b2_tr10_f8 | 32 | 829.647 ms | 942.381 ms | 0.9950 |
| b2_tr10_f8 | 48 | 1125.506 ms | 1273.767 ms | 0.9970 |
| b2_tr10_f8 | 64 | 1361.775 ms | 1505.985 ms | 0.9995 |
| b2_tr10_f8 | 96 | 1743.187 ms | 1969.979 ms | 1.0000 |
| b2_tr50_f8 | 4 | 159.156 ms | 205.735 ms | 0.9385 |
| b2_tr50_f8 | 8 | 265.862 ms | 364.709 ms | 0.9680 |
| b2_tr50_f8 | 12 | 374.703 ms | 473.330 ms | 0.9765 |
| b2_tr50_f8 | 16 | 476.989 ms | 613.124 ms | 0.9810 |
| b2_tr50_f8 | 24 | 678.687 ms | 838.559 ms | 0.9950 |
| b2_tr50_f8 | 32 | 868.990 ms | 1040.976 ms | 0.9965 |
| b2_tr50_f8 | 48 | 1202.368 ms | 1354.413 ms | 0.9990 |
| b2_tr50_f8 | 64 | 1454.398 ms | 1603.588 ms | 0.9995 |
| b2_tr50_f8 | 96 | 1896.008 ms | 2091.639 ms | 1.0000 |

Interim read: b2 is a strong recall-recovery lever at 50k f8. Both b2 cells hit
recall 1.0000 at nprobe 96, and b2/tr50 improves low-nprobe recall over
b2/tr10. Training 50k is not free: fixed-nprobe p50 is modestly slower than
training 10k at most points.

## Remaining Work

This packet does not close Task 121. Still owed:

- b4/tr10 and b4/tr50 50k recall pipelines
- full 100k Phase 2 recall matrix beyond the current b0/b1/b2 partials
- credible clean latency re-measurement on a quiesced host
- Phase 3 scan-efficiency A/B
- Phase 4 Pareto/verdict

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-50k-b2-checkpoint.md`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run.json`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run-audit.log`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run.log`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run.script.log`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run-resume.log`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run-status.log`
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json`
- `artifacts/precheck-host.log`
- `artifacts/load-50k_b2_tr10_f8.log`
- `artifacts/load-50k_b2_tr50_f8.log`
- `artifacts/load-50k_b4_tr10_f8.log`
- `artifacts/load-50k_b4_tr50_f8.log`
- `artifacts/storage-50k_b2_tr10_f8.log`
- `artifacts/storage-50k_b2_tr50_f8.log`
- `artifacts/storage-50k_b4_tr10_f8.log`
- `artifacts/storage-50k_b4_tr50_f8.log`
- `artifacts/truth-cache-50k-q200-k10.log`
- `artifacts/pipeline-50k_b2_tr10_f8.log`
- `artifacts/pipeline-50k_b2_tr50_f8.log`
