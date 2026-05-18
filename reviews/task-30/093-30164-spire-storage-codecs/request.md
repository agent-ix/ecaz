---
id: 30164
title: SPIRE Storage Codecs
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 773d7bbc
---
# Review Request: SPIRE Storage Codecs

## Summary

This checkpoint starts the Phase 1 partition-object persistence foundation
without wiring PostgreSQL relation storage yet.

The checkpoint:

- replaces the `ec_spire` storage placeholder with V1 codecs for the Phase 0
  partition-object shape
- adds discriminator-prefixed `SpireVecId` support for bounded local and global
  vector IDs
- encodes and decodes `PartitionObjectHeaderV1` fields: object kind, PID,
  object version, hierarchy level, parent PID, child count, assignment count,
  and object flags
- encodes and decodes `LeafAssignmentRowV1` fields: assignment flags,
  `vec_id`, heap TID, payload format, gamma, payload length, and encoded
  payload bytes
- validates invalid PID/version zeroes, unknown assignment flags, invalid heap
  locators, non-finite gamma, invalid `vec_id` shapes, and row-length
  mismatches
- adds focused unit tests for header, `vec_id`, and assignment-row round trips
  and rejection cases

No PostgreSQL relation-backed storage, epoch manifest publishing, build path,
scan path, insert path, or vacuum repair behavior is included in this
checkpoint.

## Files To Review

- `src/am/ec_spire/storage.rs`

## Design Alignment

This follows the Phase 0 decision record in
`plan/design/spire-phase0-partition-object-storage.md`:

- partition objects are PID-addressed index-internal objects
- leaf membership rows are logical `(vec_id, pid)` rows where the containing
  object supplies the PID
- `vec_id` is bounded to 32 encoded bytes and reserves a local/global
  discriminator
- heap TIDs are local row locators, not vector identity
- boundary-replica, tombstone, delta insert/delete, and stale-locator flags are
  represented but not behaviorally wired yet

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 10 selected tests:

- 8 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Does this byte layout match the Phase 0 storage design closely enough to
   become the durable object codec surface?
2. Are the `vec_id` discriminator and width checks adequate before relation
   storage starts allocating real local IDs?
3. Are the assignment flags modeled narrowly enough for Phase 1 while leaving
   the Phase 0 delta and boundary-replica deferrals explicit?
4. Should the header reserve additional fields before relation-backed storage
   makes the format harder to change?
