# SPIRE Root-Control Rescan Refresh

## Checkpoint

- Code commit: `7dee1793`
  (`Refresh SPIRE scan root control every rescan`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30321`

## Summary

This checkpoint addresses the review concern that the scan descriptor could
keep stale root/control cursor fields when the active epoch did not change.

The scan path already reads the root/control page on each rescan. It now also
replaces the cached `SpireRootControlState` with that observed value every
time, rather than returning the previous cached struct when `active_epoch`
matches. This removes the hidden invariant that scan-side code must never read
same-epoch cursor fields from the cached copy.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test --lib scan_opaque_refreshes_root_control_on_every_rescan_observation --no-default-features --features pg18`
  - Result: `1 passed; 0 failed; 1119 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `239 passed; 0 failed; 881 filtered out`
- `git diff --check`

## Notes

- This directly responds to packet `30321` feedback.
- The cache remains useful for diagnostics/tests, but no longer preserves stale
  same-epoch cursor fields across rescans.
