# SPIRE Leaf Maintenance Threshold Constants

## Checkpoint

- Code commit: `0cfd7b2f`
  (`Name SPIRE leaf maintenance thresholds`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for packet `30309`

## Summary

This checkpoint names the local leaf maintenance threshold policy constants in
code and updates the plan/design prose to reference those names.

The split threshold now derives from:

- `SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS`
- `SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER`

The merge threshold now derives from:

- `SPIRE_LEAF_MERGE_AVERAGE_DIVISOR`

This keeps the read-only SQL leaf diagnostic recommendations and the update
mechanics design note aligned on the same named policy knobs instead of
duplicating anonymous numeric literals.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-update-mechanics.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test leaf_maintenance_thresholds_use_named_split_merge_policy --no-default-features --features pg18`
  - `1 passed; 0 failed; 1124 filtered out`
- `cargo test --lib test_ec_spire_leaf_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 1124 filtered out`
- `git diff --check`

## Notes

- This responds to the round-review recommendation to avoid threshold formula
  drift between code and design docs.
