---
id: 30196
title: SPIRE Delete Delta Duplicate Target Guard
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 2256b3f8
---

# Review Request: SPIRE Delete Delta Duplicate Target Guard

## Summary

This checkpoint rejects duplicate delete targets inside a single SPIRE delta
draft before any new delta object is allocated.

- Tracks delete-delta `vec_id`s while validating the draft.
- Rejects a repeated `vec_id` in `delete_assignments` even when that vector is
  visible in the base snapshot.
- Adds a regression test that verifies the PID allocator, local vec-id
  allocator, and object store are unchanged when the duplicate-delete draft
  fails validation.

## Non-Goals

- No idempotent delete behavior.
- No delta merge/compaction.
- No AM callback wiring.
- No changes to visible-row overlay semantics from packet 30195.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 97 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 15 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
