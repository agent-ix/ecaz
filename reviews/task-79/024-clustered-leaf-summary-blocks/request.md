# Task 79 Review Request: Clustered Leaf Summary Blocks

## Summary

This packet reviews code commit `7cb404b5497c5b57255dbe6a619692d15564a638`.

It changes the next Task 79 selector attempt from "rank arbitrary contiguous row blocks" to "rank build-time clustered row blocks." When `ec_spire.leaf_block_rows` is enabled, the recursive build path now clusters rows inside each materialized leaf into approximately one cluster per summary block, reorders row/source-vector pairs by that subcluster, and then builds the existing V3 `SpireLeafBlockSummary` chain over those contiguous groups.

The scan path also gains `ec_spire.leaf_block_pruning_summary_radius_weight`, a bounded `0.0..1.0` RaBitQ radius-weight knob. This lets the benchmark compare mean-only, partial-radius, and full Cauchy-Schwarz-bound block ranking without changing code between rows. The default is `1.0`, preserving packet 022's full-radius behavior unless the benchmark explicitly sets another value.

## Why This Slice

Packets 015, 018, 020, 021, and 023 show the remaining failure is block admission quality:

- summary-only global pruning cuts candidates but misses recall at the 5.2M gate;
- block32 improves granularity but still needs about 8M-9.6M candidates for recall;
- sampled rows add cost and only modest recall;
- wider rerank does not change recall, proving the missing winners are not admitted into the candidate pool;
- full radius-bound global ranking is looser than summary-only and loses recall.

This slice improves the information content of the existing V3 summaries without adding a new storage format. It makes summary blocks geometrically coherent before query-time pruning scores them.

## Implementation Notes

- `layout_leaf_rows_for_block_summaries` runs only when `ec_spire.leaf_block_rows > 0`.
- V2/no-summary leaf builds are unchanged.
- Row/source-vector pairing is preserved while reordering, and the existing V3 row-base coverage validation continues to apply after layout.
- The new radius-weight GUC is scan-time only and bounded by PostgreSQL GUC validation plus a defensive helper check.

## Validation

Artifacts are under `reviews/task-79/024-clustered-leaf-summary-blocks/artifacts/`.

- `cargo fmt --check`: passed, with existing stable-toolchain warnings about nightly-only rustfmt settings.
- `cargo test -p ecaz leaf_block`: passed, 9 tests.

New focused tests:

- `leaf_block_layout_groups_rows_before_summary_chunks`
- `leaf_block_summary_radius_weight_controls_rabitq_bound`

## Review Focus

- Check whether build-time row clustering is an acceptable Task 79 summary-quality improvement without a new on-disk format.
- Check that reordering rows within a leaf does not violate row-index, vector-id, or summary coverage invariants.
- Check that the radius-weight default preserving full-radius behavior is the right compatibility choice, given packet 023's negative benchmark will be followed by explicit mean/partial-radius benchmark rows.

## Next Step

After review visibility, the next packet will run a local RaBitQ `ecaz bench suite` benchmark against the clustered V3 index. This packet does not claim a candidate or latency win by itself.
