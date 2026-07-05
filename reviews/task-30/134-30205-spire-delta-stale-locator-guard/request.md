---
id: 30205
title: SPIRE Delta Stale Locator Guard
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: c428b79b
---

# Review Request: SPIRE Delta Stale Locator Guard

## Summary

This checkpoint keeps `SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR` out of delta
partition objects.

- Rejects delta assignment rows carrying the stale-locator flag.
- Keeps the stale-locator marker available for existing leaf rows, where future
  HOT/update repair and vacuum cleanup paths can use it without changing row
  format.
- Extends invalid delta-flag coverage for a delta insert marked as stale.

## Non-Goals

- No HOT-chain repair implementation.
- No vacuum cleanup implementation.
- No scan overlay changes beyond packet 30204.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 103 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 25 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
