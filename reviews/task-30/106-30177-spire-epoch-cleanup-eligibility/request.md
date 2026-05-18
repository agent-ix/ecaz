---
id: 30177
title: SPIRE Epoch Cleanup Eligibility
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 111f29b3
---

# Review Request: SPIRE Epoch Cleanup Eligibility

## Summary

This checkpoint adds a pure retention/cleanup eligibility helper for SPIRE epoch
manifests.

- Adds `SpireEpochManifest::cleanup_eligible_at`.
- Keeps `Building` and `Published` epochs ineligible for cleanup.
- Allows `Retired` epochs to become eligible only after:
  - `now_micros >= retain_until_micros`
  - `active_query_count == 0`
- Allows `Failed` epochs to become eligible after `retain_until_micros`.
- Leaves actual object, manifest, and placement cleanup as future persistence
  work.

## Non-Goals

- No cleanup execution.
- No relation-backed manifest/object deletion.
- No retention scheduler.
- No AM callback behavior change.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 65 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 6 `ec_spire::build` unit tests
  - 30 `ec_spire::meta` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
