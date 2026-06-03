# Task 79 Packet 030: RaBitQ Multi-Representative Rank Diagnostic

This packet records a local rank diagnostic for the packet 029 multi-representative RaBitQ implementation. It explains why the best capped rows still miss the Task 79 gates.

AWS was not used.

## Result

The diagnostic row uses `global768/radius0.25`:

- 4,860,415 candidates
- p50 47.616 ms
- p95 57.164 ms
- recall@10 0.9905

The rank file has 2,000 exact top-10 targets. At cap768, 1,981 are selected and 19 are missed. The recall gate needs at least 1,985 selected targets, so this row is short by 4 exact targets, but it is already over the p50 gate.

At cap640, the same rank file selects 1,974 targets and misses 26. That is the faster candidate surface, but it needs 11 more exact targets to reach the recall gate.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, artifacts, and key result summary.
- `artifacts/leaf-block-rank-analysis.md`: cap readout and rank distribution from the JSONL file.
- `artifacts/leaf-block-rank-100k-rabitq-block32-multirep-global768-rw025.jsonl`: 2,000 exact-target rank rows.
- `artifacts/suite-status.log`: suite status, `completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `suite-rabitq-multirep-rank-diagnostic.json`: checked-in SuiteConfig used by `ecaz bench suite`.

## Interpretation

The failure is still block selection. Only 5 targets are outside routed leaves; the rest are routed but ranked below the selected cap. Multi-representative summaries helped versus single-representative block32, but not enough to make cap640 pass.

The viable next local slice is a selector/scoring change that recovers near-cap misses without raising the final cap. Cap896 reaches the recall target in rank terms, but block32 cap896 would exceed the 5.2M candidate gate.
