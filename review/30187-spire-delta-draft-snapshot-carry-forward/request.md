---
id: 30187
title: SPIRE Delta Draft Snapshot Carry-Forward
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 901824d5
---

# Review Request: SPIRE Delta Draft Snapshot Carry-Forward

## Summary

This checkpoint extends the in-memory SPIRE delta epoch draft helper so a new
delta epoch can carry forward an existing published snapshot.

- Adds `build_delta_epoch_draft_from_snapshot`.
- Validates the base snapshot before using it as the carry-forward source.
- Requires the delta object's `base_pid` to exist in the base snapshot.
- Re-epochs carried object-manifest and placement-directory entries into the
  new epoch.
- Observes carried PIDs before allocating the new delta-object PID, preventing
  PID reuse when the allocator starts behind the base snapshot.
- Keeps the standalone delta draft helper for tests and lower-level callers.

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
  - 87 selected tests passed
  - 15 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 20 `ec_spire::storage` unit tests
  - 6 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
