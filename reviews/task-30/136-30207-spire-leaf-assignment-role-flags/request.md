---
id: 30207
title: SPIRE Leaf Assignment Role Flags
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 5d77788f
---

# Review Request: SPIRE Leaf Assignment Role Flags

## Summary

This checkpoint rejects role-less rows inside leaf partition objects.

- Requires leaf object assignments to carry at least one leaf role flag:
  primary, boundary replica, tombstone, or stale locator.
- Leaves the lower-level assignment-row codec permissive so malformed rows can
  still be decoded and rejected at the object boundary.
- Removes the zero-flag row from the visible-scan fixture.
- Adds constructor/decode coverage for a leaf object containing a zero-flag row.

## Non-Goals

- No boundary-replica scan behavior changes.
- No tombstone cleanup implementation.
- No HOT-chain repair implementation.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 105 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 26 `ec_spire::storage` unit tests
  - 18 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
