# Review Request: Task 79 RaBitQ Leaf-Contrast Calibration

## Summary

Packet 036 tests a local-only RaBitQ score-calibration idea: apply a temporary scan-time `ec_spire.leaf_block_pruning_leaf_contrast_weight` that amplifies each block summary score's deviation from its routed leaf mean before global block selection. The goal was to preserve the successful global allocation from packets 029/033 while rescuing exact top-10 targets that sit just below the cap640 cutoff.

Result: negative. Contrast does not improve recall at global640; it degrades recall while holding candidate count near 4.05M. The patch is therefore kept only as `artifacts/leaf-contrast-source.patch` and should not land as production code.

## Evidence

- Packet manifest: `reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/manifest.md`
- Suite config: `reviews/task-79/036-rabitq-leaf-contrast-calibration/suite-rabitq-leaf-contrast-calibration.json`
- Compact results: `reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/compact-results.tsv`
- Raw suite output: `reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-run.log`
- Parsed results: `reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/results.jsonl`

```text
row	global_blocks	leaf_contrast_weight	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
leaf_contrast	640	0.00	0.25	4050758	44.870	52.173	0.9870	2000	fail_recall
leaf_contrast	640	0.25	0.25	4052735	45.506	51.544	0.9860	2000	fail_recall_p50
leaf_contrast	640	0.50	0.25	4053651	44.801	52.646	0.9860	2000	fail_recall
leaf_contrast	640	1.00	0.25	4054561	44.925	51.436	0.9800	2000	fail_recall
leaf_contrast	640	2.00	0.25	4055175	45.168	52.805	0.9705	2000	fail_recall_p50
leaf_contrast	640	4.00	0.25	4055601	45.277	52.652	0.9540	2000	fail_recall_p50
```

## Validation

- `cargo test --no-default-features --features pg18 leaf_block_score_contrast_amplifies_leaf_local_outliers` passed.
- `cargo test --no-default-features --features pg18 select_global_leaf_block_row_ranges` passed.
- `ecaz bench suite audit` passed.
- `ecaz bench suite status` reports `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

## Reviewer Notes

This packet closes simple per-leaf mean contrast as a direct fix. It confirms that reweighting scores within each leaf harms global block ordering rather than finding the packet 030 misses. The next local research direction should be a richer two-stage block scorer or build-time residual/radius quality calibration, not more scalar leaf-local contrast sweeps.
