# SPIRE Relation Storage Debt Diagnostics

## Checkpoint

- Code commit: `24b809b2`
  (`Expose SPIRE relation storage debt diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL diagnostics for active-referenced and cleanup-candidate
  relation-backed SPIRE object tuples

## Summary

This checkpoint makes old-epoch physical storage debt visible before tuple
reclamation is implemented:

- Added a relation object tuple scanner for SPIRE index data blocks.
- Added relation-object-store support for enumerating every tuple locator used
  by an active placement, including V2 leaf segment chains.
- Added `ec_spire_index_relation_storage_snapshot(index_oid)` as a stable,
  strict SQL table function.
- The function reports active epoch, relation block count, total relation
  object tuple count/bytes, active-referenced tuple count/bytes, and
  cleanup-candidate tuple count/bytes.
- The function reports `physical_cleanup_supported = false` so the diagnostic
  is explicit that this checkpoint observes physical debt but does not reclaim
  tuples yet.
- The focused SQL test verifies an empty index has no relation object tuples,
  a populated build has no cleanup candidates because all tuples are active
  referenced, and a post-build insert creates cleanup-candidate old-epoch
  manifest/placement tuples.
- Updated the Task 30 plan to record relation storage diagnostics while keeping
  physical reclamation and full old-epoch cleanup open.

This does not delete relation tuples, mark old line pointers unused, recycle
old relation object pages, implement retention-window cleanup, or change scan
visibility semantics.

## Changed Files

- `src/am/ec_spire/page.rs`
- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_relation_storage_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `215 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- This is not a measurement or recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Cleanup candidates are relation object tuples not referenced by the active
  root/control epoch, active object manifest, active placement directory, or
  active placement object/segment locators. They are storage-debt diagnostics,
  not a retention decision or physical deletion plan.
