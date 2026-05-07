---
id: 30200
title: SPIRE Leaf Object Vec ID Uniqueness
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 3467e70d
---

# Review Request: SPIRE Leaf Object Vec ID Uniqueness

## Summary

This checkpoint enforces Phase 1 same-leaf `vec_id` uniqueness in the leaf
partition-object codec.

- Validates leaf-object assignment lists from constructors, encoders, and
  decoders.
- Rejects duplicate `vec_id` rows within a single leaf object.
- Adds a regression test covering both constructor rejection and manually
  encoded duplicate leaf rows, including a same-leaf boundary-row duplicate to
  keep future boundary replication from weakening the `(vec_id, pid)` rule.

## Non-Goals

- No cross-PID boundary-replica behavior.
- No scan deduplication changes.
- No delta merge/compaction.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 101 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 23 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
