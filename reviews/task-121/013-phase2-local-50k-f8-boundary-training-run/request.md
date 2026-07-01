# Task 121 Phase 2 Local 50k f8 Boundary/Training Run

## Scope

This packet records a completed local-only `ecaz bench suite` continuation for the Task 121 Phase 2 50k f8 boundary/training slice.

- No AWS resources were used.
- Storage format: `rabitq`.
- PQ was not included.
- The run reused the isolated per-cell 50k tables/indexes loaded in packet 012.
- This packet is local single-PG pipeline evidence (`remote=false`), not the local multi-node lane.
- This is not task closeout evidence; 100k confirmation and later Task 121 phases remain open.

## Evidence

- Suite config: `artifacts/suite-phase2-local-50k-f8-boundary-training-run.json`
- Suite manifest: `artifacts/suite-phase2-local-50k-f8-boundary-training-run-manifest.json`
- Structured results: `artifacts/suite-phase2-local-50k-f8-boundary-training-run-results.jsonl`
- Suite log: `artifacts/suite-phase2-local-50k-f8-boundary-training-run.log`
- Script transcript: `artifacts/suite-phase2-local-50k-f8-boundary-training-run.script.log`
- Completed pipeline logs:
  - `artifacts/pipeline-50k_b0_tr50_f8.log`
  - `artifacts/pipeline-50k_b1_tr10_f8.log`
  - `artifacts/pipeline-50k_b1_tr50_f8.log`
- Compact summary: `artifacts/summary-50k-f8.txt`

The generated truth cache and per-query funnel/stage JSONLs are intentionally not committed. Aggregate metrics are captured in the structured results JSONL and pipeline logs.

## Result Summary

For comparison, packet 012 baseline `b0/tr10/f8`:

| Cell | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| b0/tr10/f8 | 8 | 113.904 ms | 160.732 ms | 0.8885 |
| b0/tr10/f8 | 24 | 336.253 ms | 446.060 ms | 0.9585 |
| b0/tr10/f8 | 32 | 455.131 ms | 588.225 ms | 0.9725 |
| b0/tr10/f8 | 48 | 735.999 ms | 849.005 ms | 0.9875 |
| b0/tr10/f8 | 64 | 979.020 ms | 1060.524 ms | 0.9950 |

This packet:

| Cell | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| b0/tr50/f8 | 8 | 119.627 ms | 163.311 ms | 0.9020 |
| b0/tr50/f8 | 24 | 340.280 ms | 467.843 ms | 0.9710 |
| b0/tr50/f8 | 32 | 455.979 ms | 610.957 ms | 0.9810 |
| b0/tr50/f8 | 48 | 757.812 ms | 888.651 ms | 0.9890 |
| b1/tr10/f8 | 8 | 203.586 ms | 260.896 ms | 0.9350 |
| b1/tr10/f8 | 24 | 511.197 ms | 602.720 ms | 0.9760 |
| b1/tr10/f8 | 32 | 697.122 ms | 796.378 ms | 0.9860 |
| b1/tr10/f8 | 48 | 977.851 ms | 1116.596 ms | 0.9925 |
| b1/tr50/f8 | 8 | 207.778 ms | 261.012 ms | 0.9480 |
| b1/tr50/f8 | 24 | 536.889 ms | 645.922 ms | 0.9895 |
| b1/tr50/f8 | 32 | 730.339 ms | 858.418 ms | 0.9945 |
| b1/tr50/f8 | 48 | 1054.866 ms | 1172.220 ms | 0.9980 |

## Interpretation

Both remaining f8 levers matter at 50k:

- Full training (`tr50`) gives a moderate recall gain at comparable latency.
- Boundary replication (`b1`) gives the larger recall gain, but roughly doubles index storage and materially increases same-nprobe latency.
- The combination (`b1/tr50/f8`) is the best 50k f8 point so far. It reaches near-baseline-high-nprobe recall at lower nprobe: `0.9945` at nprobe 32 / p50 `730.339 ms`, versus packet 012 b0/tr10/f8 `0.9950` at nprobe 64 / p50 `979.020 ms`.

This supports carrying `b1/tr50/f8` forward to a 100k local confirmation before doing any promotion or Phase 3 work.
