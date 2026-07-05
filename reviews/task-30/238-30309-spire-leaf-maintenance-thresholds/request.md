# SPIRE Leaf Maintenance Thresholds

## Checkpoint

- Code commit: `6f6c2fdf`
  (`Expose SPIRE leaf maintenance thresholds`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: read-only split/merge threshold recommendations in leaf diagnostics

## Summary

This checkpoint defines the first concrete split/merge trigger inputs without
implementing split or merge scheduling:

- Extended `ec_spire_index_leaf_snapshot(index_oid)` with split and merge
  assignment thresholds.
- Added `split_recommended`, `merge_recommended`, `maintenance_action`, and
  `maintenance_reason` columns.
- The split threshold is:
  `max(32, 4 * ceil(total_effective_assignments / active_leaf_count))`.
- The merge threshold is:
  `floor(ceil(total_effective_assignments / active_leaf_count) / 4)`.
- Empty or very sparse leaves are labeled as `merge_candidate`; leaves at or
  above the split threshold are labeled as `split_candidate`.
- Updated the Task 30 plan to record these as read-only trigger definitions
  while keeping actual schedulers open.

This does not split, merge, rebalance, rewrite leaves, or change scan routing.
It only exposes deterministic policy labels over the active leaf snapshot.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_leaf_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1098 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `218 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Thresholds are intentionally conservative and diagnostic-only.
- Split/merge execution, scheduler policy, and recall/storage evidence remain
  follow-up work.
