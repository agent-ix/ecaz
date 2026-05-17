# SPIRE Retired Epoch Residue Diagnostics

## Checkpoint

- Code commit: `8819c7b9`
  (`Harden SPIRE retired manifest diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30307`

## Summary

This checkpoint covers the partial-write residue case called out in the
`30307` review.

The epoch snapshot implementation now has a pure row-building helper so the
dedupe and blocker-label behavior can be unit tested without fabricating a
relation page. For a residue shape where root/control still points at an older
published manifest while a newer retired duplicate exists for the same epoch,
the diagnostic rows now keep the root/control manifest authoritative:

- the older root/control manifest reports `is_active_root_manifest = true`
- the older root/control manifest reports `cleanup_blocked_reason =
  active_root_manifest`, not `superseded_manifest`
- both the older root/control manifest and the newer retired residue remain
  `cleanup_eligible_now = false`

The retired-manifest writer also now rejects non-published input manifests and
documents the append-order assumption used by per-epoch manifest dedupe.

## Changed Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/mod.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test epoch_snapshot_partial_retired_residue_keeps_root_manifest_authoritative --no-default-features --features pg18`
  - `1 passed; 0 failed; 1120 filtered out`
- `cargo test --lib test_ec_spire_epoch_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 1120 filtered out`
- `git diff --check`

## Notes

- This directly addresses the first follow-up in the packet `30307` review.
- The broader `publish_replacement_epoch` helper extraction remains open.
