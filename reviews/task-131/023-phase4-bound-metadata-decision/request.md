# Task 131 Packet 023: Phase 4 Bound Metadata Decision

## Summary

Packet 022 answered the reviewer's real-scale question for the current
production-read surface: completed 10k/50k representative cells exposed no sound
threshold bounds at all. Every completed cell had `sound_bound_available_sum=0`
and zero threshold block/row availability.

I inspected the current code path and the result is expected. The threshold
profile can use SPIRE leaf block summaries as a sound RaBitQ upper-bound source,
but only when leaves have persisted summaries. The representative production-read
indexes used in Task 123/131 do not build those summaries.

## Decision

Shelve Phase 3 streaming global threshold feedback for the current default SPIRE
production-read surface.

Do not implement coordinator-to-worker threshold plumbing now. With no sound
bound metadata available, such plumbing cannot skip rows or blocks safely.

The only reasonable revival path is a separate metadata-gated experiment:
build representative multi-instance indexes with leaf block summaries enabled,
then measure whether those bounds are available, selective, storage-affordable,
and recall-safe at 10k/50k/100k for `n128/b4` and `n1024/b2`.

## Evidence

- Packet 022: `reviews/task-131/022-real-scale-threshold-boundability/`
- Current threshold profile path:
  - `src/am/ec_spire/scan/candidates.rs`
  - `collect_quantized_selected_leaf_threshold_profile`
- Existing metadata surface:
  - `src/am/ec_spire/storage/leaf_v2_parts.rs`
  - `src/am/ec_spire/options/mod.rs`
  - `ec_spire.leaf_block_rows`
  - `ec_spire.leaf_block_summary_representatives`
  - `ec_spire.leaf_block_pruning_summary_radius_weight`

## Review Ask

Please review this as the Phase 4 decision point:

- Accept shelving Phase 3 on the no-summary default surface.
- Decide whether Task 131 should proceed to a narrow leaf-summary metadata A/B,
  or move directly to Phase 5 closeout with global threshold feedback marked
  "shelved pending metadata."

This packet still does not close Task 131.
