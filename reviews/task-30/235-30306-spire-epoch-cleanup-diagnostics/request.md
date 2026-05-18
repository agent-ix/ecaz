# SPIRE Epoch Cleanup Diagnostics

## Checkpoint

- Code commit: `48d0243b`
  (`Expose SPIRE epoch cleanup diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: read-only SQL diagnostics for persisted epoch manifests and cleanup
  blockers

## Summary

This checkpoint makes epoch retention state visible before old-epoch
reclamation is implemented:

- Added `ec_spire_index_epoch_snapshot(index_oid)` as a stable, strict SQL
  table function.
- The function scans SPIRE relation object tuples for valid epoch manifest
  payloads.
- Each row reports active epoch, manifest epoch/state/consistency mode,
  publish and retention timestamps, active query count, manifest tuple locator,
  and whether the row is the active root/control manifest.
- The function applies the existing cleanup planning rules and reports
  `cleanup_eligible_now` plus a `cleanup_blocked_reason` label.
- Added focused PG18 SQL coverage for empty, populated, and post-insert
  active-epoch publication states.
- Updated the Task 30 plan to record epoch diagnostics while keeping physical
  page reclamation and old-epoch cleanup open.

This is a diagnostic checkpoint only. It does not rewrite old epoch manifests,
advance retired/failed states, remove relation object tuples, or reclaim pages.

## Changed Files

- `src/am/ec_spire/meta.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
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
- Cleanup eligibility is based on the current in-code epoch cleanup plan; it is
  not physical cleanup.
- Old published epoch rows currently remain non-cleanup-eligible until the
  follow-up state transition/reclamation path lands.
