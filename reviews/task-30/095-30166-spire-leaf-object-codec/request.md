---
id: 30166
title: SPIRE Leaf Object Codec
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 4f980e03
---
# Review Request: SPIRE Leaf Object Codec

## Summary

This checkpoint turns the existing assignment-row codec into a concrete leaf
partition-object codec. It still does not wire PostgreSQL relation persistence.

The checkpoint:

- adds prefix decoding for `LeafAssignmentRowV1` so a leaf object body can
  contain an ordered stream of rows
- adds `SpireLeafPartitionObject` with a `PartitionObjectHeaderV1` plus row
  body
- validates leaf object headers: kind must be `Leaf`, child count must be zero,
  and header assignment count must match the decoded/encoded row count
- keeps PID ownership on the containing partition object header; assignment
  rows still omit repeated PID fields
- adds tests for assignment-row prefix decode, leaf object round trip, non-leaf
  headers, child-count rejection, count mismatch, and trailing-byte rejection

No relation-backed object store, build path, scan path, insert path, or vacuum
behavior is included in this checkpoint.

## Files To Review

- `src/am/ec_spire/storage.rs`

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 25 selected tests:

- 12 `ec_spire::storage` unit tests
- 11 `ec_spire::meta` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Is the leaf object body framing adequate before the first relation-backed
   storage writer lands?
2. Should leaf objects enforce `level = 0` now, or leave level validation to the
   hierarchy/routing layer?
3. Is it acceptable that assignment rows omit PID and depend on the containing
   object header to rehydrate logical `(vec_id, pid)` membership?
