---
id: 30184
title: SPIRE Local Delta Object Store
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 3a1b3621
---

# Review Request: SPIRE Local Delta Object Store

## Summary

This checkpoint extends the in-memory SPIRE local object-store abstraction to
write and read delta partition objects.

- Adds `SpireLocalObjectStore::insert_delta_object`.
- Adds `SpireLocalObjectStore::read_delta_object`.
- Reuses local placement metadata for delta objects.
- Validates placement node/store/state before reads.
- Verifies decoded delta object PID and object version match placement
  metadata.
- Keeps relation-backed persistence out of scope.

## Non-Goals

- No PostgreSQL relation-backed object storage.
- No insert/delete AM callback behavior.
- No delta merge/compaction.
- No root/control publish transaction.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 78 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 20 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
