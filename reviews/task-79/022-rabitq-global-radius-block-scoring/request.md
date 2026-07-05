# Task 79 Review Request: RaBitQ Global Radius Block Scoring

## Summary

This code checkpoint fixes the global block pruning scorer to use the same RaBitQ block-radius upper-bound score already used by per-leaf block pruning.

Code commit under review:

- `7fe2a8de50956eaa020f5a8a80c085895eb0946f` (`Use RaBitQ block radius in global pruning`)

Files changed:

- `src/am/ec_spire/scan/candidates.rs`
- `src/am/ec_spire/scan/tests/candidates.rs`

## Change

Before this checkpoint, per-leaf block pruning ranked RaBitQ summaries with:

- `mean_ip + query_l2_norm * summary.gamma`

but global block pruning ranked the same summaries with only:

- `mean_ip`

That made the global cap more aggressive than the existing summary contract and could drop high-radius blocks whose true rows were still plausible nearest neighbors. Packet 021 showed that widening heap rerank cannot recover recall after those blocks are dropped.

This checkpoint introduces `score_leaf_block_summary_ip(...)` and routes both per-leaf and global block scoring through it. For RaBitQ summaries, the helper applies the radius bound and preserves the existing non-negative radius validation.

## Validation

Packet-local logs:

- `artifacts/cargo-fmt-check.log`: `cargo fmt --check` passed. The log includes the repo's existing rustfmt nightly-option warnings.
- `artifacts/cargo-test-leaf-block.log`: `cargo test -p ecaz leaf_block` passed.

Focused test result:

- 7 passed, 0 failed
- New regression: `select_global_leaf_block_row_ranges_uses_rabitq_summary_radius`

## Follow-Up

The next packet should benchmark the same RaBitQ global384/global512 rows from packet 021 after this scoring fix. Candidate counts should remain at the selected block budgets, so the key question is whether recall moves toward the 0.9925 gate without losing the p50 gain.
