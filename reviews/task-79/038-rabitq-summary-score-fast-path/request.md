# Review Request: Task 79 RaBitQ Summary-Score Fast Path

## Summary

Packet 038 lands a production-safe scan CPU optimization for RaBitQ leaf-block summaries. The change precomputes the summary scoring context once per selection pass and scores all zero-gamma representative payload chunks through one quantizer branch, instead of revalidating and redispatching once per representative.

Result: partial latency win, not Task 79 closure. Against the existing local k=3 index from packet 037, `global736` improves from 48.853 ms p50 to 47.909 ms p50 while preserving 4.66M candidates and 0.9925 recall@10. `global768` is essentially unchanged at 48.989 ms p50. The best row remains 2.909 ms over the 45 ms p50 target.

## Evidence

- Packet manifest: `reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/manifest.md`
- Suite config: `reviews/task-79/038-rabitq-summary-score-fast-path/suite-rabitq-summary-score-fast-path.json`
- Compact results: `reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/compact-results.tsv`
- Raw suite output: `reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-run.log`
- Parsed results: `reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/results.jsonl`

```text
row	global_blocks	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	packet037_baseline_p50_ms	p50_delta_ms	gate
fast_path	736	4657668	47.909	55.607	0.9925	2000	48.853	-0.944	fail_p50
fast_path	768	4860209	48.989	56.668	0.9925	2000	49.017	-0.028	fail_p50
```

## Validation

- `cargo test --no-default-features --features pg18 leaf_block_summary` passed: 2 tests.
- `cargo test --no-default-features --features pg18 select_global_leaf_block_row_ranges` passed: 2 tests.
- `ecaz bench suite audit` passed.
- `ecaz bench suite status` reports `completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

## Reviewer Notes

This is worth keeping as a small real optimization, but it proves scorer dispatch/validation overhead is not the remaining dominant cost. The next local axis should directly reduce candidate work further, likely by testing whether k=3/full-score at `global736` can tolerate lower routing breadth or a different index shape while preserving the 0.9925 recall row.
