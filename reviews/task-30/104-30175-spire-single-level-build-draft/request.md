---
id: 30175
title: SPIRE Single-Level Build Draft Helper
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 4f762109
---

# Review Request: SPIRE Single-Level Build Draft Helper

## Summary

This checkpoint adds an in-memory SPIRE single-level build draft helper. It
assembles the already-landed Phase 1 pieces into one validated draft without
wiring PostgreSQL relation persistence or AM build callbacks.

- Adds `SpireSingleLevelBuildInput` and `SpireSingleLevelBuildDraft`.
- Adds `build_single_level_leaf_epoch_draft`.
- Allocates a PID and local `vec_id`s through cloned allocator cursors.
- Builds primary leaf assignment rows.
- Builds a leaf partition object.
- Writes the leaf object to the local object store abstraction.
- Builds the object manifest and placement directory.
- Validates the result with `SpirePublishedEpochSnapshot`.
- Commits allocator cursors only after the complete draft validates.

## Non-Goals

- No `ambuild` or `ambuildempty` behavior change.
- No relation-backed storage or root/control publish transaction.
- No scan, insert, delete, vacuum, or replica integration.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 60 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 4 `ec_spire::build` unit tests
  - 27 `ec_spire::meta` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
