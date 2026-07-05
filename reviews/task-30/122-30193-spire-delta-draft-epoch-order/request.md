---
id: 30193
title: SPIRE Delta Draft Epoch Order
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: a66fced4
---

# Review Request: SPIRE Delta Draft Epoch Order

## Summary

This checkpoint enforces epoch ordering for delta drafts built from a published
base snapshot.

- Rejects delta drafts whose target epoch is equal to or older than the base
  snapshot epoch.
- Leaves standalone lower-level delta draft construction unchanged.
- Adds a regression test that verifies allocators and object-store state do not
  advance when the epoch-order guard rejects the draft.

## Non-Goals

- No root/control relation write.
- No global epoch registry or cross-backend coordination.
- No AM callback wiring.
- No remote/degraded placement update semantics.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 94 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 10 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
