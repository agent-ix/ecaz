# Task 79 Packet 028: Clustered Block16 Radius-Weight Benchmark

This packet records a local RaBitQ-only sweep that tests whether reducing clustered leaf blocks from 32 rows to 16 rows can directly reduce the SPIRE candidate surface enough to satisfy Task 79.

It does not pass Task 79. It is negative evidence for the finer-block tuning path.

## Result

Best under-cap row:

- `global1536/radius0.25`: 4,894,281 candidates, p50 48.613 ms, p95 55.067 ms, recall@10 0.9890.

Highest-recall row in this packet:

- `global1664/radius0.25`: 5,301,755 candidates, p50 50.025 ms, p95 58.927 ms, recall@10 0.9900.

Task 79's first accepted slice requires recall@10 >= 0.9925, candidates <= 5.2M, and p50 <= 45 ms or 25% better than the 60.256 ms baseline. No block16 row satisfies those gates.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, and key result summary.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/suite-status.log`: suite status, `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-clustered-block16-radius-weight.json`: checked-in SuiteConfig used by `ecaz bench suite`.

AWS was not used.

## Interpretation

Block16 proves the candidate surface can be brought under 5.2M, but the current single per-block summary score loses too much recall. The best under-cap row drops candidates by 68.4% versus the unbounded local baseline, yet recall falls from 0.9975 to 0.9890.

The next implementation step should directly address summary-score discrimination: add richer per-block representation, likely a multi-representative leaf-block summary, instead of continuing with block-size or cap-only tuning.
