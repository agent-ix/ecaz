---
id: 30201
title: SPIRE Leaf Object Delta Flag Guard
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: c2292629
---

# Review Request: SPIRE Leaf Object Delta Flag Guard

## Summary

This checkpoint prevents leaf partition objects from storing delta insert/delete
rows. Delta rows are now accepted only by delta partition objects.

- Validates individual leaf assignments from the leaf-object assignment-list
  validator.
- Rejects `SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT` and
  `SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE` in leaf objects.
- Adds constructor/decode coverage for a leaf object carrying a delta-insert
  row.
- Updates the scan fixture that checks non-output rows so its delete row lives
  in a delta object rather than inside the leaf object.

## Non-Goals

- No delta-object flag behavior changes.
- No scan overlay logic changes beyond fixture shape.
- No delta merge/compaction.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 102 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 24 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
