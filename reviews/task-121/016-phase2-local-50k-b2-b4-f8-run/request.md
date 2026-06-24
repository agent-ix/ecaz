# Task 121 review request: Phase 2 local 50k b2/b4 f8 matrix

## Scope

This packet responds to the reviewer gap that 50k b2/b4 f8 cells were missing.
It now covers the full local 50k b2/b4 f8 supplement:

- completed: b2/tr10 and b2/tr50 recall/storage at 50k f8
- completed: b4/tr10 recall/storage at 50k f8
- completed: b4/tr50 recall/storage at 50k f8

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

The shared manifest/status remains the b2 checkpoint view:
`completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=2`.
The b4/tr10 cell then ran as a separate `--only` resume so the user-requested
breakpoint could halt before b4/tr50:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --resume-from reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --only pipeline-50k_b4_tr10_f8 --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b4-tr10-only-resume.log
```

The b4-only status artifact reports:
`completed=1 failed=0 skipped=13 dry_run=0 missing_artifacts=0 stale=0`, with
`pipeline-50k_b4_tr10_f8` succeeded and `pipeline-50k_b4_tr50_f8` skipped.

The b4/tr50 cell then ran as a separate `--only` resume after restarting the
local PG18 cluster. The restarted cluster listened on `/tmp`, so this final
resume uses `--host /tmp`:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --resume-from reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --only pipeline-50k_b4_tr50_f8 --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b4-tr50-only-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b4-tr50-only-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b4-tr50-only-resume.log
```

The b4/tr50-only manifest reports `pipeline-50k_b4_tr50_f8` succeeded, with
the other suite steps skipped by the `--only` selector. The streamed diagnostic
files reached the expected shape: 1800 funnel rows and 10800 stage-containment
rows.

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

Completed b4/tr10 recall/latency pipeline rows:

| cell | nprobe | p50 | p95 | recall@10 |
|---|---:|---:|---:|---:|
| b4_tr10_f8 | 4 | 236.562 ms | 339.021 ms | 0.9575 |
| b4_tr10_f8 | 8 | 395.642 ms | 529.372 ms | 0.9725 |
| b4_tr10_f8 | 12 | 492.944 ms | 660.507 ms | 0.9840 |
| b4_tr10_f8 | 16 | 589.129 ms | 808.869 ms | 0.9890 |
| b4_tr10_f8 | 24 | 830.643 ms | 1032.950 ms | 0.9945 |
| b4_tr10_f8 | 32 | 1066.051 ms | 1210.029 ms | 0.9980 |
| b4_tr10_f8 | 48 | 1352.252 ms | 1523.077 ms | 0.9990 |
| b4_tr10_f8 | 64 | 1571.358 ms | 1793.543 ms | 1.0000 |
| b4_tr10_f8 | 96 | 1900.814 ms | 2199.261 ms | 1.0000 |

Completed b4/tr50 recall/latency pipeline rows:

| cell | nprobe | p50 | p95 | recall@10 |
|---|---:|---:|---:|---:|
| b4_tr50_f8 | 4 | 244.920 ms | 564.265 ms | 0.9650 |
| b4_tr50_f8 | 8 | 400.503 ms | 547.652 ms | 0.9810 |
| b4_tr50_f8 | 12 | 556.820 ms | 756.253 ms | 0.9865 |
| b4_tr50_f8 | 16 | 658.855 ms | 882.720 ms | 0.9905 |
| b4_tr50_f8 | 24 | 877.132 ms | 1107.089 ms | 0.9975 |
| b4_tr50_f8 | 32 | 1130.192 ms | 1410.404 ms | 0.9985 |
| b4_tr50_f8 | 48 | 1482.835 ms | 1740.106 ms | 1.0000 |
| b4_tr50_f8 | 64 | 1668.702 ms | 2202.986 ms | 1.0000 |
| b4_tr50_f8 | 96 | 2135.053 ms | 2647.988 ms | 1.0000 |

Interim read: b4/tr50 is the strongest 50k recall cell in this packet. It
improves low/mid-nprobe recall over b4/tr10 and reaches recall 1.0000 by
nprobe 48, earlier than b4/tr10 at nprobe 64 and both b2 cells at nprobe 96.
That recall gain is not free: b4/tr50 has b4-sized storage and higher
fixed-nprobe latency than the b2 cells, and it is slower than b4/tr10 at most
fixed nprobe points.

## Remaining Work

This packet does not close Task 121. Still owed:

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
- `artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl`
- `artifacts/suite-phase2-local-50k-b4-tr10-only-resume.log`
- `artifacts/suite-phase2-local-50k-b4-tr10-only-status.log`
- `artifacts/suite-phase2-local-50k-b4-tr10-only-manifest.json`
- `artifacts/suite-phase2-local-50k-b4-tr50-only-resume.log`
- `artifacts/suite-phase2-local-50k-b4-tr50-only-manifest.json`
- `artifacts/suite-phase2-local-50k-b4-tr50-only-results.jsonl`
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
- `artifacts/pipeline-50k_b4_tr10_f8.log`
- `artifacts/pipeline-50k_b4_tr50_f8.log`
