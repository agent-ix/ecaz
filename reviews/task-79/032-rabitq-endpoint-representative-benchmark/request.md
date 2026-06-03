# Task 79 Packet 032: RaBitQ Endpoint Representative Benchmark

This packet tests a local-only rejected variant of the RaBitQ k=2 multi-representative summary: use the farthest-pair endpoints directly instead of the two cluster means.

It does not pass Task 79. It is negative evidence: endpoint representatives keep the candidate/latency budget roughly in range but collapse recall, so the source change was reverted and is not proposed for landing.

AWS was not used.

## Result

| row | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: |
| endpoint cap512/radius0.25 | 3,228,407 | 42.414 ms | 49.493 ms | 0.8910 | 2000 |
| endpoint cap640/radius0.25 | 4,034,920 | 44.856 ms | 52.945 ms | 0.9145 | 2000 |
| endpoint cap768/radius0.25 | 4,841,459 | 47.202 ms | 56.159 ms | 0.9310 | 2000 |

For comparison, packet 029 cluster-mean k=2 at the matching caps reached recall@10 0.9795, 0.9870, and 0.9905 respectively. Endpoint reps are therefore much worse on the only failing gate.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, artifacts, and key result summary.
- `artifacts/endpoint-representative-source-diff.patch`: exact temporary source delta used for the rejected local experiment.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/suite-status.log`: suite status, `completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-endpoint-representative-block32.json`: checked-in SuiteConfig used by `ecaz bench suite`.

## Interpretation

The hypothesis was that raw endpoints might lift outlier true-neighbor blocks that cluster means were smoothing away. The result falsifies that hypothesis. Endpoints appear to over-rank block extremes and lose the stable block-level signal that made the two cluster means useful.

Next Task 79 work should stay on the accepted k=2 cluster-mean representation and focus on score calibration or a controlled richer representative strategy, not raw endpoints.
