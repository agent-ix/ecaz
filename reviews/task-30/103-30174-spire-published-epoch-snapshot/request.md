---
id: 30174
title: SPIRE Published Epoch Snapshot Validator
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 175a8a1d
---

# Review Request: SPIRE Published Epoch Snapshot Validator

## Summary

This checkpoint adds a pure metadata validator for an active SPIRE published
epoch snapshot.

- Adds `SpirePublishedEpochSnapshot` over:
  - `SpireEpochManifest`
  - `SpireObjectManifest`
  - `SpirePlacementDirectory`
- Requires the epoch manifest state to be `Published`.
- Requires epoch IDs to match across epoch, object manifest, and placement
  directory.
- Requires every object manifest PID to resolve to a placement entry.
- Requires object manifest and placement object versions to match.
- Requires strict consistency snapshots to reference only `Available`
  placements.
- Allows degraded snapshots to reference `Available`, `Unavailable`, or
  `Skipped` placements, while rejecting `Stale`.

## Non-Goals

- No relation-backed object persistence.
- No root/control publish transaction.
- No read path or scan integration.
- No replica implementation.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 56 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 27 `ec_spire::meta` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
