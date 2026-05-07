---
id: 30197
title: SPIRE Delete Delta Row Locator Validation
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: bf2ffed9
---

# Review Request: SPIRE Delete Delta Row Locator Validation

## Summary

This checkpoint tightens delete-delta validation so a tombstone target must
match the visible row locator for the `vec_id` it deletes.

- Builds a visible target map from the base snapshot's visible primary rows.
- Rejects duplicate visible `vec_id` rows while preparing delete validation.
- Rejects delete deltas whose `heap_tid` does not match the currently visible
  base row for the target `vec_id`.
- Keeps duplicate delete-target and already-deleted target guards from the
  previous checkpoints.
- Adds a regression test that confirms allocator cursors and object storage do
  not advance when a delete carries a mismatched row locator.

## Non-Goals

- No heap visibility recheck against PostgreSQL MVCC state.
- No HOT-chain or vacuum repair behavior.
- No idempotent delete behavior.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 98 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 16 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
