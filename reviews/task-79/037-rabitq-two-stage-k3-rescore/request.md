# Review Request: Task 79 RaBitQ Two-Stage K3 Rescore

## Summary

Packet 037 tests the reviewer-proposed Path B from packet 036: build RaBitQ leaf-block summaries with three cluster-mean representatives, run a cheap k=2 first-pass score for global block ranking, then rescore a shortlist with the late third representative before final block selection.

Result: negative for latency. The temporary two-stage scorer preserves the k=3 recall/candidate breakthrough, but does not lower p50 enough to close Task 79. The best row is `global736` with `full_rescore_blocks=1280`: 4.66M candidates, 0.9925 recall@10, and 48.651 ms p50. That is slightly better than full k=3 at the same cap, but still 3.651 ms over the 45 ms target and worse than the pre-k3 latency envelope.

The patch is kept only as `artifacts/two-stage-k3-rescore.patch` and should not land as production code.

## Evidence

- Packet manifest: `reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/manifest.md`
- Suite config: `reviews/task-79/037-rabitq-two-stage-k3-rescore/suite-rabitq-two-stage-k3-rescore.json`
- Compact results: `reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/compact-results.tsv`
- Raw suite output: `reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-run.log`
- Parsed results: `reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/results.jsonl`

```text
row	global_blocks	first_pass_representatives	full_rescore_blocks	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
full_k3	736	0	0	4657668	48.853	56.690	0.9925	2000	fail_p50
full_k3	768	0	0	4860209	49.017	56.424	0.9925	2000	fail_p50
two_stage	736	2	1024	4657668	49.856	60.253	0.9925	2000	fail_p50
two_stage	736	2	1280	4657668	48.651	59.354	0.9925	2000	fail_p50
two_stage	768	2	896	4860209	49.648	57.167	0.9925	2000	fail_p50
two_stage	768	2	1024	4860209	49.281	56.671	0.9925	2000	fail_p50
two_stage	768	2	1280	4860209	49.088	57.734	0.9925	2000	fail_p50
two_stage	768	2	1536	4860209	49.543	56.906	0.9925	2000	fail_p50
```

## Validation

- `cargo test --no-default-features --features pg18 leaf_block_summary_representative_limit_scores_prefix_only` passed.
- `cargo test --no-default-features --features pg18 full_rescore_promotes_block_with_late_representative_hit` passed.
- `cargo test --no-default-features --features pg18 rabitq_leaf_block_summary_records_three_cluster_representatives` passed.
- `ecaz bench suite audit` passed.
- `ecaz bench suite status` reports `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

## Reviewer Notes

This closes the conservative k=2-first-pass/k=3-rescore axis in its current form. The main useful finding is that k=3 can hit the recall gate at `global736` with fewer candidates than packet 035's `global768`, but the p50 remains around 48.7-48.9 ms. The next local axis should target the scan CPU cost of full k=3 scoring directly, or a build-time k=2-compatible summary/calibration that avoids scan-time late-representative work.
