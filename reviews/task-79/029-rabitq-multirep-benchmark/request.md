# Task 79 Packet 029: RaBitQ Multi-Representative Benchmark

This packet records the local RaBitQ benchmark for the first direct summary-discrimination implementation: V4 RaBitQ leaf-block summaries with two representatives per block.

It does not pass Task 79. It is useful negative evidence: the richer summaries improve the best capped recall/candidate tradeoff versus the prior block32 sweep, but not enough to meet the accepted-slice gates.

## Result

Best row with candidate and p50 gates satisfied:

- `global640/radius0.25`: 4,050,758 candidates, p50 44.852 ms, p95 52.726 ms, recall@10 0.9870.

Highest-recall capped row:

- `global768/radius0.25`: 4,860,415 candidates, p50 48.670 ms, p95 55.112 ms, recall@10 0.9905.

Task 79's first accepted slice requires recall@10 >= 0.9925, candidates <= 5.2M, and p50 <= 45 ms or 25% better than the 60.256 ms baseline. No multi-representative block32 row satisfies those gates.

TurboQuant was not run because RaBitQ is the primary/default target and did not pass.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, validation, and key result summary.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/suite-status.log`: suite status, `completed=9 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/cargo-check-pg18.log`: PG18 feature `cargo check`; passed.
- `artifacts/cargo-test-leaf-block.log`: focused leaf block scoring/coverage tests; 10 passed.
- `artifacts/cargo-test-leaf-partition-object.log`: focused storage round-trip/rejection tests; 14 unit tests and 2 fixture tests passed.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-multirep-block32.json`: checked-in SuiteConfig used by `ecaz bench suite`.

AWS was not used.

## Interpretation

The multi-representative implementation did reduce the candidate surface at comparable recall versus the prior single-representative block32 sweep, but it did not recover enough recall under the latency gate. The strongest capped row still needs another 0.20 percentage points of recall and is 3.67 ms over the p50 gate.

The next local step should be a multi-representative false-negative/rank diagnostic: compare target block ranks under the new score to packet 026's rank tail and identify whether the miss is caused by score calibration, representative selection, radius handling, or a selection-policy issue.
