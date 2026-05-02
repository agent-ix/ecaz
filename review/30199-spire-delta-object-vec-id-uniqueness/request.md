---
id: 30199
title: SPIRE Delta Object Vec ID Uniqueness
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: cb010126
---

# Review Request: SPIRE Delta Object Vec ID Uniqueness

## Summary

This checkpoint moves the duplicate `vec_id` invariant into the delta
partition-object codec so malformed object bytes cannot bypass draft-builder
validation.

- Adds delta-object assignment-list validation that checks each row's delta
  flags and rejects duplicate `vec_id`s.
- Applies the same validation from constructors, encoders, and decoders.
- Adds a regression test covering both constructor rejection and manually
  encoded duplicate delta rows.

## Non-Goals

- No leaf-object duplicate-row policy change.
- No boundary-replica semantics.
- No delta merge/compaction.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 100 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 22 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
