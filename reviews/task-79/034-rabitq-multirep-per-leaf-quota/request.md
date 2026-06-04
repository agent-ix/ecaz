# Task 79 Packet 034: RaBitQ Multi-Representative Per-Leaf Quota

This packet tests whether the remaining RaBitQ miss pattern is caused by global cap leaf imbalance. It keeps the accepted RaBitQ V4 two-cluster-mean block summaries, disables the global block cap, and selects a fixed quota per routed leaf.

It does not pass Task 79. Per-leaf quotas lower latency, but recall collapses.

AWS was not used.

## Result

| per-leaf blocks | candidates | p50 | p95 | recall@10 | returned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 3,014,842 | 36.654 ms | 42.298 ms | 0.8390 | 2000 |
| 6 | 3,610,773 | 37.284 ms | 43.540 ms | 0.8720 | 2000 |
| 7 | 4,196,212 | 38.813 ms | 44.785 ms | 0.8930 | 2000 |
| 8 | 4,772,824 | 40.898 ms | 47.476 ms | 0.9130 | 2000 |

Best recall row is per-leaf 8: candidates and p50 are inside the target band, but recall is 0.9130 versus the 0.9925 gate.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, artifacts, and key result summary.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/suite-status.log`: suite status, `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-multirep-per-leaf-quota.json`: checked-in SuiteConfig used by `ecaz bench suite`.

## Interpretation

This is negative evidence for per-leaf quota as the Task 79 fix. The candidate reduction is real, but it is blunt: even 8 blocks per routed leaf loses too many exact top-10 targets.

The next RaBitQ work should target block-score discrimination or summary quality directly. The most promising local follow-up is a richer cluster-mean summary experiment, for example k=3 representatives per leaf block, because the current k=2/global-cap path is close on latency/candidates but remains short on recall.
