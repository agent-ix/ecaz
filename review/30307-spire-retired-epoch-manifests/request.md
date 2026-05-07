# SPIRE Retired Epoch Manifests

## Checkpoint

- Code commit: `e6b115ba`
  (`Retire previous SPIRE epoch manifests`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: previous-active epoch state transition during replacement epoch
  publication

## Summary

This checkpoint moves replacement publishes one step closer to old-epoch
cleanup without reclaiming physical tuples:

- Added a helper that writes a retired copy of the previous active epoch
  manifest to the SPIRE relation object store.
- Post-build insert delta publishes now write that retired manifest copy before
  advancing root/control to the new active epoch.
- Vacuum delete-delta and delta-compaction replacement publishes do the same.
- `ec_spire_index_epoch_snapshot(index_oid)` now handles duplicate manifest
  tuples for the same epoch by using the newest tuple per epoch for cleanup
  planning and labeling older rows as `superseded_manifest`.
- Focused SQL coverage now verifies a post-insert replacement epoch exposes
  two logical epochs, one retired previous epoch manifest, one active root
  manifest, and one superseded manifest row.
- Updated the Task 30 plan to record retired previous-epoch manifests while
  keeping physical old-epoch reclamation open.

This does not remove object tuples, delete old placement/object manifests, mark
old line pointers unused, or implement retention-window reclamation. It only
records the epoch state transition needed before those cleanup steps can be
made durable.

## Changed Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/vacuum.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_epoch_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `217 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- The old published manifest tuple remains on disk and is visible as
  `superseded_manifest` in the epoch snapshot.
- Physical page reclamation, object tuple deletion, SQL VACUUM end-to-end
  harness coverage, and retention-window enforcement remain follow-up work.
