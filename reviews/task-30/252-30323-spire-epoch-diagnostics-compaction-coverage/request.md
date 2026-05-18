# SPIRE Epoch Diagnostics Compaction Coverage

## Checkpoint

- Code commit: `283fe13d`
  (`Cover SPIRE epoch diagnostics after compaction`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Epoch diagnostics coverage for replacement-epoch retired manifests

## Summary

This checkpoint extends epoch diagnostics SQL coverage through vacuum
compaction:

- The existing `ec_spire_index_epoch_snapshot` PG18 test already covered empty,
  populated, and post-insert active-epoch publication states.
- The test now runs no-delete vacuum cleanup after a post-build insert delta,
  forcing compaction into a replacement V2 base leaf epoch.
- It verifies the epoch snapshot now reports five persisted manifest rows
  across three distinct epochs.
- It verifies two retired manifest rows exist: one for the post-insert publish
  and one for the vacuum-compaction replacement publish.
- It verifies superseded manifest labels rise to two while the active root
  manifest advances to epoch `3`.
- Updated the Task 30 plan summary to record post-vacuum-compaction retired
  manifest coverage.

This is coverage only. It does not change epoch retention, cleanup
eligibility, physical reclamation, or the epoch snapshot SQL shape.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_epoch_snapshot_sql --no-default-features --features pg18 -- --nocapture`
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
- Physical reclamation remains deferred; this packet only proves the persisted
  epoch manifest diagnostics see replacement-epoch retired copies.
