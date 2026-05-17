---
id: 30185
title: SPIRE Delta Assignment Builders
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 4c55dc26
---

# Review Request: SPIRE Delta Assignment Builders

## Summary

This checkpoint adds assignment-row builders for SPIRE delta objects.

- Adds an insert-delta assignment builder that allocates local `vec_id` values
  and sets `PRIMARY | DELTA_INSERT`.
- Adds a delete-delta assignment builder that reuses the caller-provided
  `vec_id` and sets `TOMBSTONE | DELTA_DELETE`.
- Shares validation for allocator-backed assignment rows.
- Validates generated insert/delete rows against `SpireDeltaPartitionObject`.
- Keeps relation-backed persistence, AM callback behavior, and delta compaction
  out of scope.

## Non-Goals

- No PostgreSQL relation-backed object storage.
- No insert/delete AM callback wiring.
- No root/control publish transaction.
- No delta merge/compaction.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 81 selected tests passed
  - 15 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 20 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
