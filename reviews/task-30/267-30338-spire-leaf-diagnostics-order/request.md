# SPIRE Leaf Diagnostics Order Safety

## Checkpoint

- Code commit: `f22ecfb3`
  (`Preserve SPIRE leaf delta diagnostics order`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30308`

## Summary

This checkpoint fixes the review-reported leaf diagnostics ordering bug.

Previously, `index_leaf_snapshot` accumulated delta counters into a stub row
when a Delta manifest entry appeared before its parent Leaf, but the later Leaf
branch replaced the whole row and zeroed those counters. The base leaf path now
updates leaf-specific fields in place and preserves any prior delta counts and
delta bytes for the same leaf PID.

The fix protects downstream consumers that use leaf snapshot output:

- split/merge recommendation diagnostics
- insert-batching debt diagnostics
- operator-facing per-leaf effective assignment counts

## Changed Files

- `src/am/ec_spire/mod.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test --lib leaf_snapshot_base_row_preserves_prior_delta_counts --no-default-features --features pg18`
  - Result: `1 passed; 0 failed; 1119 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `239 passed; 0 failed; 881 filtered out`
- `git diff --check`

## Notes

- This addresses the per-packet `30308` feedback and the inherited downstream
  concern for `30309` and `30310`.
- It does not change current manifest write order; it makes diagnostics
  correct if future split/merge publication produces Delta-before-Leaf order.
