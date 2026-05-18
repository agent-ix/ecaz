---
id: 30189
title: SPIRE Visible Delta Scan Overlay
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 19ff8210
---

# Review Request: SPIRE Visible Delta Scan Overlay

## Summary

This checkpoint applies collected delta rows to the in-memory visible-primary
scan collector.

- Treats delete-delta rows as `vec_id` tombstones for visible output.
- Preserves visible base leaf rows that are not tombstoned by a delete delta.
- Includes visible insert-delta rows in the returned scan rows.
- Keeps boundary-replica and tombstone filtering in the existing visible-row
  predicate.
- Extends the mixed leaf/delta snapshot test to verify delete suppression and
  insert visibility.

## Non-Goals

- No AM scan callback wiring.
- No heap visibility checks beyond stored row locators.
- No scoring/rerank integration.
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
