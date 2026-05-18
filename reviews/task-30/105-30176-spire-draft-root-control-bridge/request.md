---
id: 30176
title: SPIRE Draft Root-Control Bridge
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 997f3b41
---

# Review Request: SPIRE Draft Root-Control Bridge

## Summary

This checkpoint adds the pure bridge from a validated single-level SPIRE build
draft to root/control metadata.

- Adds `SpirePublishedManifestLocators`.
- Adds `SpireSingleLevelBuildDraft::root_control_state`.
- Revalidates the draft through `SpirePublishedEpochSnapshot`.
- Produces `SpireRootControlState::published` with:
  - active epoch from the draft epoch manifest
  - next PID cursor from the draft
  - next local `vec_id` cursor from the draft
  - externally supplied manifest locators
- Rejects invalid manifest locators through existing root/control validation.

## Non-Goals

- No manifest persistence.
- No root/control publish transaction.
- No AM callback behavior change.
- No relation-backed object storage.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 62 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 6 `ec_spire::build` unit tests
  - 27 `ec_spire::meta` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
