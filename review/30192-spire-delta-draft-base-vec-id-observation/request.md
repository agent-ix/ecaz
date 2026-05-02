---
id: 30192
title: SPIRE Delta Draft Base Vec ID Observation
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 52081acc
---

# Review Request: SPIRE Delta Draft Base Vec ID Observation

## Summary

This checkpoint prevents stale local `vec_id` allocators from reusing IDs when
building a delta epoch from an existing published snapshot.

- Scans carried base-snapshot leaf and delta objects for assignment `vec_id`s.
- Observes those IDs before allocating insert-delta rows.
- Keeps carried PID observation in the same draft transaction path.
- Adds a regression test where the caller intentionally passes stale PID and
  local-`vec_id` allocators.

## Non-Goals

- No root/control relation read.
- No remote/degraded placement update semantics.
- No AM callback wiring.
- No global `vec_id` rewrite implementation.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 93 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 9 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
