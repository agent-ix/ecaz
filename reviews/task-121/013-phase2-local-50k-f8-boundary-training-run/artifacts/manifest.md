# Manifest: Task 121 Phase 2 Local 50k f8 Boundary/Training Run

- Head SHA: `dcd68e8c2b1b1aab24d83bef9229f808d95ca675`
- Task bucket: `reviews/task-121/`
- Packet path: `reviews/task-121/013-phase2-local-50k-f8-boundary-training-run/`
- Timestamp: `2026-06-23T15:35:00-07:00`
- Host/lane: local PG18 on `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench_task121`
- Runner: `target/debug/ecaz bench suite run`
- Suite: `task121-phase2-local-50k-f8-boundary-training-run`
- Corpus/query scale: staged real corpus 50k, 200 queries, k=10
- Storage format: `rabitq`
- PQ: not included
- AWS: not used
- Table/index isolation: reused the one-table/index-per-cell surfaces loaded in packet 012
- Distributed mode: no; this packet used the local single-PG pipeline path (`remote=false`)

## Command

```bash
script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/013-phase2-local-50k-f8-boundary-training-run/artifacts/suite-phase2-local-50k-f8-boundary-training-run.json --manifest-output reviews/task-121/013-phase2-local-50k-f8-boundary-training-run/artifacts/suite-phase2-local-50k-f8-boundary-training-run-manifest.json --results-output reviews/task-121/013-phase2-local-50k-f8-boundary-training-run/artifacts/suite-phase2-local-50k-f8-boundary-training-run-results.jsonl --log-file reviews/task-121/013-phase2-local-50k-f8-boundary-training-run/artifacts/suite-phase2-local-50k-f8-boundary-training-run.log" reviews/task-121/013-phase2-local-50k-f8-boundary-training-run/artifacts/suite-phase2-local-50k-f8-boundary-training-run.script.log
```

## Inputs And Axes

- Config: `suite-phase2-local-50k-f8-boundary-training-run.json`
- Pre-run audit: `suite-phase2-local-50k-f8-boundary-training-run-audit.log`
- Boundary replica count: `0`, `1`
- Training sample rows: `10000`, `50000`
- Recursive fanout: `8`
- `nlists=128`
- Top graph enabled with degree `32`, build list `100`, search list `96`
- Pipeline nprobe sweep: `4,8,12,16,24,32,48,64,96`

The baseline `b0/tr10/f8` was completed and packaged in packet 012. This packet measures the three remaining f8 cells needed to isolate training, boundary replication, and their interaction at 50k:

- `b0/tr50/f8`
- `b1/tr10/f8`
- `b1/tr50/f8`

## Artifacts

- `suite-phase2-local-50k-f8-boundary-training-run.json`: checked suite config.
- `suite-phase2-local-50k-f8-boundary-training-run-audit.log`: pre-run audit output.
- `suite-phase2-local-50k-f8-boundary-training-run.log`: suite runner log.
- `suite-phase2-local-50k-f8-boundary-training-run.script.log`: command transcript.
- `suite-phase2-local-50k-f8-boundary-training-run-manifest.json`: suite runner manifest.
- `suite-phase2-local-50k-f8-boundary-training-run-results.jsonl`: structured result rows.
- `pipeline-50k_b0_tr50_f8.log`: completed pipeline recall/latency summary.
- `pipeline-50k_b1_tr10_f8.log`: completed pipeline recall/latency summary.
- `pipeline-50k_b1_tr50_f8.log`: completed pipeline recall/latency summary.
- `summary-50k-f8.txt`: compact summary of the three completed pipeline cells.
- `truth-cache-50k-q200-k10.log`: truth-cache generation log.
- `truth-cache-50k-q200-k10.json`: generated truth cache, intentionally not committed.
- `pipeline-50k_*-funnel.jsonl`: generated per-query funnel diagnostics, intentionally not committed.
- `pipeline-50k_*-stage-containment.jsonl`: generated per-stage containment diagnostics, intentionally not committed.

## Key Result Lines

For comparison, packet 012 baseline `b0/tr10/f8`:

| Cell | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| b0/tr10/f8 | 4 | 60.686 ms | 83.550 ms | 0.8190 |
| b0/tr10/f8 | 8 | 113.904 ms | 160.732 ms | 0.8885 |
| b0/tr10/f8 | 24 | 336.253 ms | 446.060 ms | 0.9585 |
| b0/tr10/f8 | 32 | 455.131 ms | 588.225 ms | 0.9725 |
| b0/tr10/f8 | 48 | 735.999 ms | 849.005 ms | 0.9875 |
| b0/tr10/f8 | 64 | 979.020 ms | 1060.524 ms | 0.9950 |

This packet:

| Cell | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| b0/tr50/f8 | 4 | 62.340 ms | 86.481 ms | 0.8210 |
| b0/tr50/f8 | 8 | 119.627 ms | 163.311 ms | 0.9020 |
| b0/tr50/f8 | 24 | 340.280 ms | 467.843 ms | 0.9710 |
| b0/tr50/f8 | 32 | 455.979 ms | 610.957 ms | 0.9810 |
| b0/tr50/f8 | 48 | 757.812 ms | 888.651 ms | 0.9890 |
| b0/tr50/f8 | 64 | 1028.760 ms | 1141.052 ms | 0.9960 |
| b1/tr10/f8 | 4 | 114.603 ms | 161.841 ms | 0.8920 |
| b1/tr10/f8 | 8 | 203.586 ms | 260.896 ms | 0.9350 |
| b1/tr10/f8 | 24 | 511.197 ms | 602.720 ms | 0.9760 |
| b1/tr10/f8 | 32 | 697.122 ms | 796.378 ms | 0.9860 |
| b1/tr10/f8 | 48 | 977.851 ms | 1116.596 ms | 0.9925 |
| b1/tr10/f8 | 64 | 1225.481 ms | 1380.495 ms | 0.9975 |
| b1/tr50/f8 | 4 | 114.398 ms | 152.131 ms | 0.9020 |
| b1/tr50/f8 | 8 | 207.778 ms | 261.012 ms | 0.9480 |
| b1/tr50/f8 | 24 | 536.889 ms | 645.922 ms | 0.9895 |
| b1/tr50/f8 | 32 | 730.339 ms | 858.418 ms | 0.9945 |
| b1/tr50/f8 | 48 | 1054.866 ms | 1172.220 ms | 0.9980 |
| b1/tr50/f8 | 64 | 1294.813 ms | 1417.144 ms | 0.9990 |

Storage was captured in packet 012 for these already-loaded cells:

| Cell family | Index size |
| --- | ---: |
| b0 | 40.6-40.7 MiB |
| b1 | 79.7-79.8 MiB |

## Interpretation

`training_sample_rows=50000` is a real but moderate 50k improvement at `b0/f8`: recall moves from `0.8885` to `0.9020` at nprobe 8, `0.9585` to `0.9710` at nprobe 24, and `0.9725` to `0.9810` at nprobe 32. Latency is roughly comparable.

`boundary_replica_count=1` is the larger recall lever at 50k, but it is expensive at equal nprobe. At `tr10/f8`, nprobe 8 moves from `0.8885` to `0.9350`, but p50 moves from `113.904 ms` to `203.586 ms` and index size roughly doubles.

The combined `b1/tr50/f8` cell is the best recall/cost point observed in the 50k f8 slice. It reaches `0.9945` at nprobe 32 with p50 `730.339 ms`, close to the b0/tr10/f8 nprobe 64 recall of `0.9950` at p50 `979.020 ms`, while using about twice the index storage. It also reaches `0.9895` at nprobe 24, slightly above b0/tr10/f8 nprobe 48 recall `0.9875`, with lower p50 (`536.889 ms` vs `735.999 ms`).

This is still not Phase 2 closeout: the 100k confirmation is missing, and the local multi-node lane remains distinct from this local single-PG pipeline measurement.
