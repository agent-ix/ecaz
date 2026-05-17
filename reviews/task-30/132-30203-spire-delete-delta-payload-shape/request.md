---
id: 30203
title: SPIRE Delete Delta Payload Shape
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 76889d36
---

# Review Request: SPIRE Delete Delta Payload Shape

## Summary

This checkpoint canonicalizes delete-delta assignment rows so tombstones carry
only vector identity and row locator data.

- Requires delete-delta rows to use `payload_format = 0`.
- Requires delete-delta rows to use `gamma = 0.0`.
- Requires delete-delta rows to carry an empty encoded payload.
- Updates storage and scan fixtures to use canonical delete-delta rows.
- Adds regression coverage for non-zero delete payload format, non-zero gamma,
  and non-empty delete payload bytes.

## Non-Goals

- No insert-delta payload policy changes.
- No heap visibility recheck.
- No delta merge/compaction.
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
