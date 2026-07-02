# Task 121 Phase 2 Local 50k/100k b0/b1 Checkpoint

## Scope

This packet records a partial local-only checkpoint for the Task 121 Phase 2 b0/b1 drill after packet 011 showed b2/b4 were too costly at 10k.

- No AWS resources were used.
- Storage format: `rabitq`.
- PQ was not included.
- The run used isolated one table/index per cell.
- This packet is not task closeout evidence.
- This checkpoint is local single-PG pipeline evidence (`remote=false`), not the local multi-node lane.

## Evidence

- 50k suite config: `artifacts/suite-phase2-local-50k-b0-b1-run.json`
- 100k suite config prepared/audited but not run: `artifacts/suite-phase2-local-100k-b0-b1-run.json`
- Suite manifest: `artifacts/suite-phase2-local-50k-b0-b1-run-manifest.json`
- Suite logs/transcripts:
  - `artifacts/suite-phase2-local-50k-b0-b1-run.log`
  - `artifacts/suite-phase2-local-50k-b0-b1-run.script.log`
  - `artifacts/suite-phase2-local-50k-b0-b1-run-resume.log`
  - `artifacts/suite-phase2-local-50k-b0-b1-run-resume.script.log`
- Per-cell load/storage logs are under `artifacts/`.
- Completed pipeline logs:
  - `artifacts/pipeline-50k_b0_tr10_f8.log`
  - `artifacts/pipeline-50k_b0_tr10_f16.log`
- Compact summary: `artifacts/partial-50k-summary.txt`

The generated truth cache and per-query funnel/stage JSONLs are intentionally not committed. The partial third pipeline cell (`b0_tr50_f8`) is not cited.

## Result Summary

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

50k storage was captured for all eight b0/b1 cells:

| Cell family | Index size |
| --- | ---: |
| b0 | 40.6-40.7 MiB |
| b1 | 79.7-79.8 MiB |

## Interpretation

The 50k baseline is materially harder than 10k. The b0/tr10/f8 cell starts at `0.8190` recall@10 at nprobe 4 and only reaches `0.9950` at nprobe 64.

The completed f8-vs-f16 comparison does not support fanout 16 as an independent winner at 50k baseline. It is slightly worse at low nprobe and effectively neutral at high nprobe, with similar or worse latency.

This run was stopped after the second completed pipeline cell to avoid continuing a long partial matrix blindly under a nearly full local filesystem. Remaining Phase 2 work is to complete the 50k boundary/training cells and then run the selected 100k evidence locally.
