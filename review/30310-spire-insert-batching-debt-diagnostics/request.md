# SPIRE Insert Batching Debt Diagnostics

## Checkpoint

- Code commit: `fb3fdf6c`
  (`Expose SPIRE insert batching debt diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL diagnostics for post-build insert delta fanout and batching debt

## Summary

This checkpoint exposes insert-batching debt before implementing the actual
batching path:

- Added `ec_spire_index_insert_debt_snapshot(index_oid)` as a stable, strict
  SQL table function.
- The function reports active epoch, active leaf count, leaf count with deltas,
  total delta object count, total delta insert assignment count, and max delta
  objects per leaf.
- It reports `insert_batching_supported = false` and a
  `batching_recommended` flag when repeated post-build inserts create multiple
  delta objects for the same active leaf.
- The recommendation text points at batching inserts by routed base leaf before
  replacement-epoch publication.
- Existing repeated same-leaf post-build insert coverage now verifies the
  diagnostic reports three delta objects on one leaf and recommends batching.
- Updated the Task 30 plan to make the insert batching gap visible in SQL while
  keeping the implementation open.

This does not add `aminsertcleanup`, command-local pending insert state, or a
batched publish path. It only makes the current per-row epoch fanout explicit.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_after_build_multiple_same_leaf_deltas --no-default-features --features pg18 -- --nocapture`
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
- Insert batching remains follow-up work.
- The diagnostic is intentionally active-epoch scoped and does not inspect
  old/superseded epoch objects.
