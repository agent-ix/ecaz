# Review Request: Task 79 RaBitQ Sampled Global Block Pruning

## Summary

This packet reviews commit `2e3d12f71d1d2f3a04ce5425dacffafc7f2f13c3`, which adds the next direct candidate-surface reduction attempt for Task 79.

Packet 015 showed that summary-only global block allocation improves latency only at unacceptable recall: the candidate-gate runs stayed at recall@10 0.9675-0.9710, while the first recall-floor point still scanned 9.44M candidates. This patch keeps the global block budget path but adds an optional sampled probe stage:

- `ec_spire.leaf_block_pruning_global_probe_blocks`
- `ec_spire.leaf_block_pruning_sample_rows_per_block`

When both are enabled and the probe block count is larger than `ec_spire.leaf_block_pruning_max_global_blocks`, SPIRE first retains a larger summary-ranked global block frontier, scores deterministic sample rows from those blocks, appends those sampled rows through the normal candidate accumulator, and then applies the final global block cap using the sampled block scores.

Important accounting point: sampled rows are not hidden. They call the normal visible candidate observer and accumulator path before the final block scan, so benchmark `candidate_sum` includes the extra sampled row scoring. Duplicate rows from finally selected blocks go through the existing dedupe/truncation accounting.

## Code Shape

- Default behavior remains unchanged unless the new GUCs are set.
- The RaBitQ-only guard remains in place.
- Leaves with summaries but no selected global block still scan zero rows, preserving the packet 014 invariant.
- Summary-only global pruning is retained as the fallback when sampling is disabled.

## Validation

Artifacts are under `reviews/task-79/016-rabitq-sampled-global-block-pruning/artifacts/`.

- `cargo fmt --check`
- `cargo test -p ecaz sampled_global_leaf_block_row_ranges_reranks_probe_blocks`
- `cargo test -p ecaz select_global_leaf_block_row_ranges_spends_budget_across_leaves`
- `cargo test -p ecaz collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer`

All focused tests passed.

## Review Focus

- Check that sampled rows are honestly accounted as candidates and are not a hidden scoring phase.
- Check that the zero-scan semantics for unselected summarized leaves are preserved.
- Check that the new knobs are disabled by default and cannot affect TurboQuant or non-RaBitQ scans.
- Check whether appending sampled rows as real candidates before final block scanning is the right contract for the Task 79 benchmark.

## Follow-Up

If this code review passes, the next packet should run an `ecaz bench suite` sweep on the RaBitQ-primary Task 79 fixture. The likely first sweep is final blocks around 320-448, probe blocks around 768-1536, and sample rows 1-4 per block.
