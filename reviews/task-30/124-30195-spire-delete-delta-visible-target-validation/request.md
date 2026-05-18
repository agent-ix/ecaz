---
id: 30195
title: SPIRE Delete Delta Visible Target Validation
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 6a77d935
---

# Review Request: SPIRE Delete Delta Visible Target Validation

## Summary

This checkpoint refines delete-delta validation to target currently visible
base rows, not every historical assignment row carried by a snapshot.

- Keeps observing all base assignment `vec_id`s for allocator safety.
- Validates delete-delta targets against visible primary rows after the base
  snapshot's existing delta overlay is applied.
- Rejects repeated deletes of an already tombstoned `vec_id`.
- Adds a regression test for a two-delta sequence where the second delete
  targets a vector deleted by the first delta.

## Non-Goals

- No idempotent delete behavior.
- No heap visibility checks beyond stored row locators.
- No AM callback wiring.
- No delta merge/compaction.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 96 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 14 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
