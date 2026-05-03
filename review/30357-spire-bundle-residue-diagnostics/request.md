# 30357 SPIRE Bundle Residue Diagnostics

## Request

Review the synthetic epoch-diagnostics coverage for a partial publish where
the new manifest bundle was written but root/control was not advanced.

## Scope

- Added `epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative`.
- Updated Task 30 status to record partial-publish retired/bundle residue
  coverage.

## Behavior Covered

The test builds an epoch snapshot with:

- epoch 7 published manifest still referenced by root/control
- epoch 7 retired residue written after the published manifest
- epoch 8 published manifest residue from a bundle write that did not become
  active

It asserts the epoch 7 root/control manifest remains
`is_active_root_manifest = true` and cleanup-blocked by
`active_root_manifest`, while the epoch 8 bundle residue is not active and is
not cleanup eligible as a published, non-root manifest.

This complements the existing retired-written residue test and covers the
other partial-publish state called out in the 30307/round review.

## Validation

- `cargo fmt`
- `cargo test epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative --no-default-features --features pg18`
- `git diff --check`

