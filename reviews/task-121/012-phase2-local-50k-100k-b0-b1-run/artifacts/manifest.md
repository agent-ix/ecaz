# Manifest: Task 121 Phase 2 Local 50k/100k b0/b1 Checkpoint

- Head SHA: `b43b9ab1dbf7146ef7d2e66a62daf156ff90a220`
- Task bucket: `reviews/task-121/`
- Packet path: `reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/`
- Timestamp: `2026-06-23T13:45:00-07:00`
- Host/lane: local PG18 on `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench_task121`
- Runner: `target/debug/ecaz bench suite run`
- Suite: `task121-phase2-local-50k-b0-b1-run`
- Corpus/query scale: staged real corpus 50k, 200 queries, k=10
- Storage format: `rabitq`
- PQ: not included
- AWS: not used
- Table/index isolation: one table/index per cell
- Distributed mode: no; this checkpoint used the local single-PG pipeline path (`remote=false`)

## Commands

Initial 50k run:

```bash
script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run.json --manifest-output reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-manifest.json --results-output reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-results.jsonl --log-file reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run.log" reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run.script.log
```

Resume after local disk-full recovery:

```bash
script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run.json --resume-from reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-manifest.json --manifest-output reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-manifest.json --results-output reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-results.jsonl --log-file reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-resume.log" reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/artifacts/suite-phase2-local-50k-b0-b1-run-resume.script.log
```

## Inputs And Axes

- 50k config: `suite-phase2-local-50k-b0-b1-run.json`
- 100k config prepared/audited only: `suite-phase2-local-100k-b0-b1-run.json`
- Pre-run audits:
  - `suite-phase2-local-50k-b0-b1-run-audit.log`
  - `suite-phase2-local-100k-b0-b1-run-audit.log`
- Boundary replica count: `0`, `1`
- Training sample rows: `10000`, `50000`
- Recursive fanout: `8`, `16`
- `nlists=128`
- Top graph enabled with degree `32`, build list `100`, search list `96`
- Pipeline nprobe sweep: `4,8,12,16,24,32,48,64,96`

## Artifacts

- `suite-phase2-local-50k-b0-b1-run.json`: checked 50k suite config.
- `suite-phase2-local-100k-b0-b1-run.json`: checked 100k suite config, not run in this packet.
- `suite-phase2-local-50k-b0-b1-run-audit.log`: 50k pre-run audit output.
- `suite-phase2-local-100k-b0-b1-run-audit.log`: 100k pre-run audit output.
- `suite-phase2-local-50k-b0-b1-run.log`: initial suite runner log.
- `suite-phase2-local-50k-b0-b1-run.script.log`: initial command transcript.
- `suite-phase2-local-50k-b0-b1-run-resume.log`: resumed suite runner log.
- `suite-phase2-local-50k-b0-b1-run-resume.script.log`: resumed command transcript.
- `suite-phase2-local-50k-b0-b1-run-manifest.json`: suite runner manifest. The suite was stopped after two completed pipeline cells.
- `load-50k_*.log`: load/build logs for all eight 50k b0/b1 cells.
- `storage-50k_*.log`: storage reports for all eight 50k b0/b1 cells.
- `pipeline-50k_b0_tr10_f8.log`: completed pipeline recall/latency summary.
- `pipeline-50k_b0_tr10_f16.log`: completed pipeline recall/latency summary.
- `partial-50k-summary.txt`: compact summary of the two completed pipeline cells.
- `truth-cache-50k-q200-k10.log`: truth-cache generation log.
- `truth-cache-50k-q200-k10.json`: generated truth cache, intentionally not committed.
- `pipeline-50k_*-funnel.jsonl`: generated per-query funnel diagnostics, intentionally not committed.
- `pipeline-50k_*-stage-containment.jsonl`: generated per-stage containment diagnostics, intentionally not committed.

The consolidated `suite-phase2-local-50k-b0-b1-run-results.jsonl` was not emitted because the suite was intentionally stopped before all pipeline steps completed.

## Suite State

Completed:

- All 50k load/storage steps for b0/b1 x tr10/tr50 x f8/f16.
- `truth-cache-50k-q200-k10`.
- `pipeline-50k_b0_tr10_f8`.
- `pipeline-50k_b0_tr10_f16`.

Pending/not complete:

- `pipeline-50k_b0_tr50_f8` was started then interrupted; its partial JSONLs are not committed or cited.
- `pipeline-50k_b0_tr50_f16`.
- `pipeline-50k_b1_tr10_f8`.
- `pipeline-50k_b1_tr10_f16`.
- `pipeline-50k_b1_tr50_f8`.
- `pipeline-50k_b1_tr50_f16`.
- 100k was not run.

## Key Result Lines

Completed 50k baseline cells:

| Cell | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| b0/tr10/f8 | 4 | 60.686 ms | 83.550 ms | 0.8190 |
| b0/tr10/f8 | 8 | 113.904 ms | 160.732 ms | 0.8885 |
| b0/tr10/f8 | 16 | 232.978 ms | 309.152 ms | 0.9370 |
| b0/tr10/f8 | 32 | 455.131 ms | 588.225 ms | 0.9725 |
| b0/tr10/f8 | 48 | 735.999 ms | 849.005 ms | 0.9875 |
| b0/tr10/f8 | 64 | 979.020 ms | 1060.524 ms | 0.9950 |
| b0/tr10/f8 | 96 | 1503.223 ms | 1588.310 ms | 1.0000 |
| b0/tr10/f16 | 4 | 65.536 ms | 107.334 ms | 0.8100 |
| b0/tr10/f16 | 8 | 115.288 ms | 159.151 ms | 0.8860 |
| b0/tr10/f16 | 16 | 229.069 ms | 295.289 ms | 0.9390 |
| b0/tr10/f16 | 32 | 465.147 ms | 560.268 ms | 0.9750 |
| b0/tr10/f16 | 48 | 738.968 ms | 821.850 ms | 0.9870 |
| b0/tr10/f16 | 64 | 992.713 ms | 1123.922 ms | 0.9945 |
| b0/tr10/f16 | 96 | 1494.388 ms | 1629.581 ms | 0.9995 |

50k ec_spire index sizes:

| Cell | Index size | Per-row bytes |
| --- | ---: | ---: |
| b0/tr10/f8 | 40.7 MiB | 852.6 B |
| b0/tr10/f16 | 40.7 MiB | 854.4 B |
| b0/tr50/f8 | 40.6 MiB | 852.0 B |
| b0/tr50/f16 | 40.7 MiB | 853.4 B |
| b1/tr10/f8 | 79.7 MiB | 1671.2 B |
| b1/tr10/f16 | 79.8 MiB | 1673.0 B |
| b1/tr50/f8 | 79.7 MiB | 1671.3 B |
| b1/tr50/f16 | 79.8 MiB | 1672.8 B |

## Interpretation

The 50k baseline is far from the 10k near-ceiling regime: b0/tr10/f8 is only `0.8190` recall@10 at nprobe 4 and needs nprobe 64 to reach `0.9950`. This confirms that routing containment gets materially harder at 50k.

`recursive_fanout=16` does not improve the completed 50k b0/tr10 baseline. It is slightly worse at low nprobe (`0.8100` vs `0.8190` at nprobe 4) and essentially neutral at high nprobe (`0.9995` vs `1.0000` at nprobe 96), with similar or worse latency. That supports de-prioritizing fanout as an independent winner unless the remaining boundary/training cells show an interaction.

This packet is not Phase 2 completion evidence. It is a checkpoint preserving completed 50k load/storage/truth and two completed baseline pipeline cells before continuing the local-only drill.
