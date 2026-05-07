# SPIRE Replacement Epoch Publish Helper

## Checkpoint

- Code commit: `8d8bc2e6`
  (`Factor SPIRE replacement epoch publish`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30307`

## Summary

This checkpoint folds the repeated replacement-publish sequence into
`publish_replacement_epoch_to_relation`.

The helper now owns the shared order for replacement epoch publication:

1. encode and validate the new manifest bundle
2. append the retired copy of the previous epoch manifest
3. append the new manifest bundle
4. derive and publish the new root/control state

The existing insert-delta, vacuum delete-delta, and vacuum delta-compaction
publish paths now call that helper instead of duplicating the sequence.

## Changed Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/vacuum.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_epoch_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 1120 filtered out`
- `cargo test --lib ec_spire_vacuum --no-default-features --features pg18 -- --nocapture`
  - `4 passed; 0 failed; 1117 filtered out`
- `git diff --check`

## Notes

- This directly addresses the second follow-up in the packet `30307` review.
- Bootstrap publishes still use the existing direct manifest publish sequence
  because they have no previous active epoch manifest to retire.
