# SPIRE Storage Debt Compaction Coverage

## Checkpoint

- Code commit: `2137b840`
  (`Cover SPIRE storage debt after compaction`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Relation-storage diagnostics coverage for replacement-epoch cleanup
  debt

## Summary

This checkpoint extends the relation-storage diagnostics SQL coverage through
the vacuum compaction path:

- The existing `ec_spire_index_relation_storage_snapshot` PG18 test already
  covered empty indexes, populated active-reference accounting, and post-insert
  cleanup-candidate tuples.
- The test now runs no-delete vacuum cleanup after a post-build insert delta,
  forcing compaction into a replacement V2 base leaf epoch.
- It verifies the storage snapshot active epoch advances to `3`.
- It verifies relation object tuple count, cleanup-candidate tuple count, and
  cleanup-candidate bytes all grow after compaction.
- Updated the Task 30 plan summary to record cleanup-debt coverage for both
  insert-delta and vacuum-compaction replacement epochs.

This is coverage only. It does not implement physical old-epoch reclamation,
change retention rules, or alter the relation-storage diagnostic SQL shape.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_relation_storage_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `232 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Physical reclamation remains deferred; this packet only proves the
  diagnostic counts the compaction-created cleanup debt.
