---
id: 30198
title: SPIRE Delta Leaf Base Guard
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 27bf25d0
---

# Review Request: SPIRE Delta Leaf Base Guard

## Summary

This checkpoint enforces Phase 1 parent-PID semantics for delta partition
objects: a delta draft built from a snapshot must target a leaf partition
object, not another delta object.

- Looks up the `base_pid` placement before carrying the base snapshot forward.
- Reads the base object header and requires `SpirePartitionObjectKind::Leaf`.
- Rejects a draft that tries to publish a delta with a prior delta PID as its
  base.
- Adds a regression test that verifies no allocator cursor or object-store page
  is advanced by the rejected nested-delta draft.

## Non-Goals

- No recursive/internal partition-object support.
- No delta merge/compaction.
- No AM callback wiring.
- No remote placement behavior.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 99 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
