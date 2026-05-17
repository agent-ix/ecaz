---
id: 30171
title: SPIRE Primary Assignment Builder
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 0f2584d8
---
# Review Request: SPIRE Primary Assignment Builder

## Summary

This checkpoint adds the helper that turns heap locators and encoded vector
payloads into primary leaf assignment rows with allocated local `vec_id`s.

The checkpoint:

- adds `SpireLeafAssignmentInput`
- adds `build_primary_leaf_assignments`
- validates heap TID, finite gamma, and payload length before consuming a local
  `vec_id`
- emits `SpireLeafAssignmentRow` values with `PRIMARY` set
- preserves input order while allocating monotonic local `vec_id`s
- adds `observe_assignment_vec_ids` for rebuilding allocator state from decoded
  assignment rows
- adds focused tests for row construction, validation without allocator
  advancement, and allocator reconstruction from assignment rows

No centroid routing, build callback wiring, insert callback wiring, relation
storage, scan scoring, or vacuum behavior is included in this checkpoint.

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

The focused test command passed 42 selected tests:

- 8 `ec_spire::assign` unit tests
- 17 `ec_spire::meta` unit tests
- 15 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Should primary row construction live in `assign.rs`, or closer to the build
   writer once centroid routing lands?
2. Is validation-before-allocation the right contract for failed row inputs?
3. Are the row inputs sufficient before quantizer-specific payload builders are
   wired in?
