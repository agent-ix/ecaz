---
id: 30172
title: SPIRE Root Control State Codec
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 6ce92369
---
# Review Request: SPIRE Root Control State Codec

## Summary

This checkpoint adds the root/control state codec for SPIRE metadata. It
records the active epoch, allocation cursors, and locators for the active
metadata blobs, without writing a PostgreSQL root/control page yet.

The checkpoint:

- adds `SpireRootControlState`
- represents an empty index with `active_epoch = 0`, invalid active manifest
  locators, `next_pid = 1`, and `next_local_vec_seq = 1`
- represents a published state with valid active epoch, epoch manifest locator,
  object manifest locator, and placement directory locator
- rejects zero PID and local `vec_id` cursors
- rejects empty states that reference active manifests
- rejects active states missing any required active manifest/directory locator
- adds a fixed root/control header with magic, metadata format version, and
  reserved bytes
- adds focused tests for empty/published round trips, cursor validation,
  manifest locator validation, and corrupt header rejection

No root/control relation page write, WAL, active-epoch publish operation,
cleanup, build callback, scan callback, insert callback, or vacuum behavior is
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

The focused test command passed 46 selected tests:

- 8 `ec_spire::assign` unit tests
- 21 `ec_spire::meta` unit tests
- 15 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Is `active_epoch = 0` the right empty-index sentinel?
2. Should root/control store locators for separate epoch/object/placement
   blobs, or should the next publish slice collapse those into one root tuple?
3. Are allocator cursors in root/control enough for Phase 1 before real
   concurrent allocation is wired?
