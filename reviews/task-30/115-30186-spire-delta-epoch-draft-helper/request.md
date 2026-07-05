---
id: 30186
title: SPIRE Delta Epoch Draft Helper
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 41279a7d
---

# Review Request: SPIRE Delta Epoch Draft Helper

## Summary

This checkpoint adds the first in-memory update-path draft helper for
epoch-published SPIRE delta objects.

- Adds `SpireDeltaEpochInput` and `SpireDeltaEpochDraft`.
- Builds insert and delete delta assignment rows into one delta partition
  object.
- Allocates a new PID for the delta object while preserving external allocator
  state on failed drafts.
- Writes the delta object through the local object store and records matching
  object/placement manifests.
- Validates the resulting metadata with `SpirePublishedEpochSnapshot`.
- Rejects empty delta drafts so update publication cannot create a no-op delta
  object.

## Non-Goals

- No PostgreSQL relation-backed object storage.
- No `aminsert` or vacuum callback wiring.
- No delta application during scans.
- No delta merge/compaction or split/merge trigger behavior.
- No root/control publish transaction.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 85 selected tests passed
  - 15 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 20 `ec_spire::storage` unit tests
  - 4 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
