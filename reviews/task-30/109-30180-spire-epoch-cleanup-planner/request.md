---
id: 30180
title: SPIRE Epoch Cleanup Planner
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: cc776719
---

# Review Request: SPIRE Epoch Cleanup Planner

## Summary

This checkpoint adds a deterministic cleanup planner for SPIRE epoch manifests.
It applies the retention rules without deleting objects, manifests, or
placements.

- Adds `SpireEpochCleanupPlan`.
- Adds `plan_epoch_cleanup`.
- Validates manifests before planning cleanup.
- Rejects duplicate epoch IDs.
- Never cleans the active epoch.
- Keeps the newest `SPIRE_MAX_RETAINED_RETIRED_EPOCHS` retired epochs.
- Cleans eligible older retired epochs only after retention and active-query
  checks pass.
- Cleans eligible failed epochs after `retain_until_micros`.

## Non-Goals

- No cleanup execution.
- No relation-backed object/manifest deletion.
- No background retention scheduler.
- No AM callback behavior change.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 71 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 6 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
