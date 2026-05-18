---
id: 30194
title: SPIRE Delete Delta Target Validation
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 849a3077
---

# Review Request: SPIRE Delete Delta Target Validation

## Summary

This checkpoint validates delete-delta targets when building a delta epoch from
a published base snapshot.

- Builds a set of observed base-snapshot assignment `vec_id`s.
- Rejects delete-delta rows whose `vec_id` is absent from that base snapshot.
- Keeps standalone lower-level delta draft construction unchanged.
- Adds a regression test proving the failed draft does not advance allocators
  or write a new object.

## Non-Goals

- No idempotent delete semantics.
- No heap visibility checks beyond stored row locators.
- No AM callback wiring.
- No remote/degraded placement update semantics.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 95 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 13 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
