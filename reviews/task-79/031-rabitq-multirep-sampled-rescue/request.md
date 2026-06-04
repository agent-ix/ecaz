# Task 79 Packet 031: RaBitQ Multi-Representative Sampled Rescue

This packet tests whether the existing sampled global selector can rescue the near-cap misses exposed by packet 030 while keeping the final cap at 640 blocks.

It does not pass Task 79. It is negative evidence for sampled rescue on the multi-representative RaBitQ path.

AWS was not used.

## Result

| row | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: |
| summary-only cap640/radius0.25 | 4,050,758 | 45.272 ms | 53.010 ms | 0.9870 | 2000 |
| probe896/sample1/prior0.8 | 4,229,958 | 49.407 ms | 57.603 ms | 0.9870 | 1933 |
| probe1024/sample1/prior0.8 | 4,255,558 | 49.596 ms | 57.262 ms | 0.9870 | 1933 |
| probe896/sample2/prior0.8 | 4,409,158 | 50.826 ms | 60.268 ms | 0.9870 | 1878 |

No sampled row improves recall, and all sampled rows worsen p50 and returned-row completeness.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, artifacts, and key result summary.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/suite-status.log`: suite status, `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-multirep-sampled-rescue.json`: checked-in SuiteConfig used by `ecaz bench suite`.

## Interpretation

Packet 030 showed cap640 needs 11 more exact top-10 targets to reach the recall gate. The sampled selector does not recover any of them in the measured rows. It also adds enough scoring overhead to move p50 well outside the latency gate and causes under-return.

The next Task 79 implementation should not continue with sampled rescue. The remaining viable local path is a scoring/selection change that reorders the 640 final blocks directly, or a cheaper richer summary representation that recovers near-cap targets without sampling row payloads.
