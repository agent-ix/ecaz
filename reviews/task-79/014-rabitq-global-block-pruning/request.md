# Review Request: Task 79 RaBitQ Global Leaf Block Pruning

## Summary

This packet requests review for commit `fc2b6ca022ba9e6384807ea2c791c6a784b4a034` (`Add RaBitQ global leaf block pruning`).

The code adds a disabled-by-default RaBitQ global leaf-block pruning path:

- new GUC: `ec_spire.leaf_block_pruning_max_global_blocks`
- after routed leaf grouping/prefetch, RaBitQ scans with the global GUC enabled load routed V2/V3 leaves, score all leaf block summaries once, retain the top K summary blocks across all routed leaves, and scan only selected row ranges
- per-leaf block pruning remains unchanged when the global GUC is `0`
- global selection uses mean-only RaBitQ summary scores intentionally; packet 013 accepted radius-adjusted selection as negative evidence
- weak routed leaves with summaries receive an explicit empty selected range when none of their blocks are in the global top K, so they scan zero leaf rows instead of falling back to full-leaf scans
- delta rows still flow through the existing loaded-delta and accumulator path

This is a direct candidate-surface reduction hook for the next benchmark. It does not claim Task 79 success until an `ecaz bench suite` packet proves recall, candidate count, and latency.

## Validation

Artifacts are under `reviews/task-79/014-rabitq-global-block-pruning/artifacts/`.

- `cargo fmt --check` passed (`artifacts/cargo-fmt-check.log`)
- `cargo test -p ecaz select_global_leaf_block_row_ranges_spends_budget_across_leaves` passed (`artifacts/cargo-test-global-selector.log`)
- `cargo test -p ecaz collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer` passed (`artifacts/cargo-test-quantized-scan.log`)

## Review Focus

- Check that the global path preserves existing default behavior when `ec_spire.leaf_block_pruning_max_global_blocks = 0`.
- Check that global selection does not accidentally full-scan leaves that have summaries but no selected block.
- Check delta handling in the global path against the existing per-leaf scan ordering and delete filtering.
- Check whether the V2 read requirement under the global GUC is the right failure mode for this experimental RaBitQ path.
