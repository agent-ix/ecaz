---
id: 30170
title: SPIRE Local Vec ID Allocator
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: f67b733e
---
# Review Request: SPIRE Local Vec ID Allocator

## Summary

This checkpoint adds the index-local `vec_id` allocator for Phase 1. It keeps
the Phase 0 identity decision executable without wiring PostgreSQL persistence
or build callbacks yet.

The checkpoint:

- adds `SpireLocalVecIdAllocator`
- starts local vector IDs at sequence 1
- allocates discriminator-prefixed `SpireVecId::local(...)` values
- rejects invalid persisted allocator state with next sequence 0
- detects local sequence exhaustion without advancing allocator state
- can observe existing local `vec_id`s and advance the next sequence during
  rebuild/reload
- ignores global `vec_id`s when reconstructing local allocator state
- adds focused unit tests for allocation, observation, invalid state, global-ID
  ignore, and exhaustion behavior

No root/control persistence, concurrent allocation, build callback, insert
callback, remote/global ID rewrite, or relation storage is included in this
checkpoint.

## Files To Review

- `src/am/ec_spire/assign.rs`

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 39 selected tests:

- 5 `ec_spire::assign` unit tests
- 17 `ec_spire::meta` unit tests
- 15 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Should local sequence 0 remain reserved/invalid, or should it be a valid
   first local vector ID?
2. Is observing existing rows enough for allocator reconstruction before root
   metadata persistence lands?
3. Does ignoring global IDs match the local-to-global rewrite story from Phase
   0?
