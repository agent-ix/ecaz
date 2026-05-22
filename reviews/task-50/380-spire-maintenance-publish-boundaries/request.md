# Review Request: SPIRE Maintenance Publish Boundaries

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `b2a070479e8f1281c10a0cc74eb2a79cc2a5bd3c`

## Summary

This slice centralizes SPIRE publish-lock acquisition and consolidates split-maintenance heap setup.

- Added `SpireLiveIndexRelation::publish_lock()` so maintenance and epoch cleanup code acquire the publish lock through the typed live-relation wrapper instead of repeated raw relation callsites.
- Updated maintenance planning, maintenance run planning, maintenance execution, and epoch cleanup to call `index.publish_lock()`.
- Consolidated split scheduled-replacement setup: heap relation open, active maintenance snapshot, and indexed vector attribute resolution now share one documented boundary.
- No safe raw-pointer helper signatures were added.

Unsafe count movement:

- `src`: 1164 -> 1159 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_spire/coordinator/maintenance.rs src/am/ec_spire/coordinator/snapshots.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1159 current unsafe rows`.

Artifacts are in `reviews/task-50/380-spire-maintenance-publish-boundaries/artifacts/`.
