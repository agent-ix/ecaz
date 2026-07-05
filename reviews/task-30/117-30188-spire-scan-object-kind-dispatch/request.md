---
id: 30188
title: SPIRE Scan Object-Kind Dispatch
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 4dbb41c0
---

# Review Request: SPIRE Scan Object-Kind Dispatch

## Summary

This checkpoint lets the in-memory scan collectors distinguish leaf and delta
partition objects inside the same published snapshot.

- Adds `SpireLocalObjectStore::read_object_header` for object-kind dispatch
  without fully decoding the object as a leaf or delta first.
- Refactors local object reads through a shared raw-object byte reader.
- Updates `collect_snapshot_leaf_rows` to skip non-leaf partition objects.
- Adds `SpireDeltaScanRow` and `collect_snapshot_delta_rows`.
- Tests a carried-forward snapshot containing both a base leaf object and a
  delta object.

## Non-Goals

- No delta overlay/application into visible scan rows.
- No AM scan callback wiring.
- No PostgreSQL relation-backed object storage.
- No delta merge/compaction.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 89 selected tests passed
  - 15 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 6 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
