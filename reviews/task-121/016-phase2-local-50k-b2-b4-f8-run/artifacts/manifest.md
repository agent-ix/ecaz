# Task 121 packet 016 artifact manifest

## Packet

- Task bucket: `reviews/task-121/`
- Packet path: `reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/`
- Head SHA: `14a44f0128a7c6def3f706dc1794ec16110f5335`
- Timestamp: 2026-06-23 17:47-21:08 America/Los_Angeles
- Lane: Task 121 Phase 2 local 50k f8 b2/b4 supplement
- Fixture: local staged real corpus, 50k corpus, 200-query pipeline sweep
- Storage format / quantizer: RaBitQ storage with TurboQuant f8 route-stage
  candidate surface, `bits=4`, `profile=ec_spire`
- Rerank mode: default pipeline rerank path; no explicit rerank-width override
- Isolation: local single PostgreSQL instance, one index per table
- Remote / multi-node: `remote=false`; this packet is not AWS evidence and is
  not the local multi-node Phase 0 lane

## Commands

Audit:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json
```

Initial run:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.log
```

Resume from the interrupted run:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --resume-from reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-resume.log
```

Status capture:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite status --manifest reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-status.log
```

B4/tr10-only resume from the breakpoint:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run.json --resume-from reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --only pipeline-50k_b4_tr10_f8 --manifest-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-manifest.json --results-output reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b2-b4-f8-run-results.jsonl --log-file reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/artifacts/suite-phase2-local-50k-b4-tr10-only-resume.log
```

## Artifacts

- `suite-phase2-local-50k-b2-b4-f8-run.json`
  - SuiteConfig for b2/b4 load, storage, truth, and pipeline steps.
  - Pipeline sweep: `4,8,12,16,24,32,48,64,96`
  - Query limit: 200
- `suite-phase2-local-50k-b2-b4-f8-run-audit.log`
  - Audit result: `[suite:task121-phase2-local-50k-b2-b4-f8-run] audit passed: 14 steps`
- `precheck-host.log`
  - PostgreSQL 18.3, `ecaz_build_profile=release`, adaptive nprobe off.
- `suite-phase2-local-50k-b2-b4-f8-run.log`
  - Initial suite runner log through the interrupted b2/tr50 step.
- `suite-phase2-local-50k-b2-b4-f8-run.script.log`
  - Terminal capture for the initial suite run; command exited 130 after the
    user-requested WSL pause.
- `suite-phase2-local-50k-b2-b4-f8-run-resume.log`
  - Resume runner log. The resume reused completed steps, finished
    `pipeline-50k_b2_tr50_f8`, then was halted at the next breakpoint after
    starting the b4/tr10 step.
- `suite-phase2-local-50k-b2-b4-f8-run-status.log`
  - Status after the breakpoint halt: completed=12, failed=0, pending=2.
- `suite-phase2-local-50k-b2-b4-f8-run-manifest.json`
  - Structured suite manifest. Succeeded: precheck, all b2/b4 load and storage
    steps, truth cache, `pipeline-50k_b2_tr10_f8`, and
    `pipeline-50k_b2_tr50_f8`. Pending: both b4 pipeline steps.
- `suite-phase2-local-50k-b2-b4-f8-run-results.jsonl`
  - Structured extracted results for the b4/tr10-only resume.
- `suite-phase2-local-50k-b4-tr10-only-resume.log`
  - Resume runner log for `--only pipeline-50k_b4_tr10_f8`.
- `suite-phase2-local-50k-b4-tr10-only-status.log`
  - B4-only status: completed=1, failed=0, skipped=13.
- `suite-phase2-local-50k-b4-tr10-only-manifest.json`
  - B4-only suite manifest preserved before restoring the shared b2 checkpoint
    manifest.
- `load-50k_b2_tr10_f8.log`, `load-50k_b2_tr50_f8.log`,
  `load-50k_b4_tr10_f8.log`, `load-50k_b4_tr50_f8.log`
  - Local one-index-per-table corpus load logs.
- `storage-50k_b2_tr10_f8.log`, `storage-50k_b2_tr50_f8.log`,
  `storage-50k_b4_tr10_f8.log`, `storage-50k_b4_tr50_f8.log`
  - Storage evidence for all four b2/b4 f8 cells.
- `truth-cache-50k-q200-k10.log`
  - Truth-cache generation log. The generated truth-cache JSON is intentionally
    untracked per packet hygiene rules.
- `pipeline-50k_b2_tr10_f8.log`
  - Completed b2/tr10 pipeline recall and funnel summary.
- `pipeline-50k_b2_tr50_f8.log`
  - Completed b2/tr50 pipeline recall and funnel summary.
- `pipeline-50k_b4_tr10_f8.log`
  - Completed b4/tr10 pipeline recall and funnel summary.
- `summary-50k-b2-checkpoint.md`
  - Compact interpretation table for this interim b2 plus b4/tr10 checkpoint.

## Key Result Lines

Storage:

| cell | index | per-row index | table total |
|---|---:|---:|---:|
| b2_tr10_f8 | 118.7 MiB | 2488.7 B | 913.6 MiB |
| b2_tr50_f8 | 118.8 MiB | 2490.4 B | 913.7 MiB |
| b4_tr10_f8 | 196.9 MiB | 4129.6 B | 991.9 MiB |
| b4_tr50_f8 | 196.9 MiB | 4128.8 B | 991.8 MiB |

Completed 50k b2 pipeline cells:

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

Completed 50k b4/tr10 pipeline cell:

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

## Interpretation

The completed b2 cells close the 50k b2 evidence gap for f8 recall/storage.
The b4/tr10 cell adds the first b4 recall result at 50k: it reaches recall
1.0000 by nprobe 64, but costs substantially more storage and fixed-nprobe
latency than b2/tr10. B4/tr50 recall remains pending because the user requested
a halt at the b4/tr10 benchmark breakpoint.
