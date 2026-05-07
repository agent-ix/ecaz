---
id: 30173
title: SPIRE PID Allocator
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: ec51a94a
---

# Review Request: SPIRE PID Allocator

## Summary

This checkpoint adds the SPIRE partition-id allocator used by later build and
publish code to assign index-internal partition object PIDs.

- Adds `SpirePidAllocator` in `src/am/ec_spire/assign.rs`.
- Moves `SPIRE_FIRST_PID` into the assignment/allocation module and reuses it
  from root/control metadata.
- Starts PID allocation at `1`; PID `0` remains invalid/sentinel space.
- Rejects `next_pid = 0` and observed PID `0`.
- Observes existing PIDs without rewinding allocator state.
- Detects sequence exhaustion without advancing state.

## Non-Goals

- No relation-backed object persistence.
- No build, scan, publish, or delete path changes.
- No replica implementation.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 50 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 21 `ec_spire::meta` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
