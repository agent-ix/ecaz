---
id: 30167
title: SPIRE Placement Directory Codec
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 2834b6af
---
# Review Request: SPIRE Placement Directory Codec

## Summary

This checkpoint adds a persisted byte shape for the per-epoch PID placement
directory. It builds on the placement-entry codec but still does not write the
directory to PostgreSQL relation pages.

The checkpoint:

- adds `SpirePlacementDirectory` as a deterministic sequence of
  `SpirePlacementEntry` rows for one epoch
- sorts entries by PID during construction
- enforces one placement entry per PID within an epoch
- rejects entry epochs that do not match the directory epoch
- adds a fixed directory header with magic, metadata format version, reserved
  bytes, epoch, and entry count
- adds binary lookup by PID for scan/build code that will consume the directory
- adds tests for round trip, sorting, PID lookup, duplicate rejection, epoch
  mismatch rejection, corrupt header rejection, and length mismatch rejection

No active-epoch root/control storage, relation-backed placement writes,
publication, cleanup, build path, scan path, or degraded-mode behavior is
included in this checkpoint.

## Files To Review

- `src/am/ec_spire/meta.rs`

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 28 selected tests:

- 14 `ec_spire::meta` unit tests
- 12 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Is one placement entry per PID per epoch the right Phase 1 invariant, given
   future replicas are explicitly deferred?
2. Should the directory codec preserve input order, or is sorted-by-PID the
   right durable shape for deterministic lookup and review?
3. Is a standalone placement-directory blob acceptable for the first
   root/control implementation, or should placement rows only ever be stored as
   individual tuples?
