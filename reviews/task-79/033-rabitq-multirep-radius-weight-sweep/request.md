# Task 79 Packet 033: RaBitQ Multi-Representative Radius-Weight Sweep

This packet follows the packet 029/032 reviewer recommendation to sweep radius weight on the accepted RaBitQ k=2 cluster-mean representation at cap640.

It does not pass Task 79. It is negative evidence for scalar radius-weight tuning as the remaining recall fix.

AWS was not used.

## Result

| radius weight | candidates | p50 | p95 | recall@10 | returned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0.00 | 4,015,761 | 45.697 ms | 54.352 ms | 0.9840 | 2000 |
| 0.10 | 4,035,744 | 45.529 ms | 52.734 ms | 0.9850 | 2000 |
| 0.20 | 4,046,743 | 44.858 ms | 50.847 ms | 0.9865 | 2000 |
| 0.25 | 4,050,758 | 44.696 ms | 53.143 ms | 0.9870 | 2000 |
| 0.30 | 4,054,052 | 45.145 ms | 53.398 ms | 0.9865 | 2000 |
| 0.40 | 4,059,747 | 44.786 ms | 51.076 ms | 0.9840 | 2000 |
| 0.50 | 4,063,715 | 44.973 ms | 53.222 ms | 0.9810 | 2000 |

Best row is radius weight 0.25: candidates and p50 pass, recall remains 0.9870 versus the 0.9925 gate.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, artifacts, and key result summary.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/suite-status.log`: suite status, `completed=9 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-multirep-radius-weight-sweep.json`: checked-in SuiteConfig used by `ecaz bench suite`.

## Interpretation

The recall curve is shallow and peaks at the existing 0.25 setting. Larger radius weights begin to hurt recall. This closes radius-weight tuning as the cheap recovery path for the 11 missed targets at cap640.

Next Task 79 work should stay local and target block-score discrimination directly: per-leaf or per-block score calibration, or a controlled richer cluster-mean summary that adds signal without raw endpoint noise.
