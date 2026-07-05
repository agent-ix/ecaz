# SPIRE Leaf Partition Diagnostics

## Checkpoint

- Code commit: `9764713a`
  (`Expose SPIRE leaf partition diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: per-leaf SQL diagnostics for split/merge trigger inputs

## Summary

This checkpoint exposes active leaf partition accounting before implementing
split/merge scheduling:

- Added `ec_spire_index_leaf_snapshot(index_oid)` as a stable, strict SQL table
  function.
- The function reports one row per active leaf PID.
- Each row includes parent PID, object version, local store identity, placement
  state, base assignment count, delta object count, delta insert/delete
  assignment counts, effective assignment count, and leaf/delta object bytes.
- Delta objects are attributed back to their base leaf via the delta object's
  parent PID.
- The diagnostic gives future split and merge triggers concrete row-count and
  storage inputs without implementing either scheduler.
- Updated the Task 30 plan to record the SQL leaf diagnostics surface while
  keeping actual split/merge thresholds open.

This does not split, merge, rebalance, or rewrite leaf partitions. It only
reports the active snapshot shape needed to define those policies.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
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
- Split and merge thresholds remain follow-up work.
- The function reports active relation-backed partition objects only; remote
  placement and boundary replication remain deferred.
