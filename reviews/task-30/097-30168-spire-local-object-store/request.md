---
id: 30168
title: SPIRE Local Object Store
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: aa169d95
---
# Review Request: SPIRE Local Object Store

## Summary

This checkpoint adds a local page-chain store scaffold for encoded SPIRE leaf
partition objects. It uses the shared `DataPageChain` primitive as the
relation-page stand-in, but does not wire access-method callbacks to real
PostgreSQL relation I/O yet.

The checkpoint:

- adds `SpireLocalObjectStore`
- appends encoded `SpireLeafPartitionObject` bytes to a `DataPageChain`
- returns a Phase 1 single-store `SpirePlacementEntry` for each stored object
- validates non-zero store relid and epoch before storing
- reads a leaf object back through its placement entry
- rejects reads for non-local nodes, non-zero local store IDs, mismatched store
  relids, non-available placement states, mismatched PID/object version, and
  mismatched object byte lengths
- adds focused tests for write/read round trip, invalid store/epoch rejection,
  and mismatched placement rejection

No PostgreSQL buffer/page manager integration, WAL, active-epoch publication,
build callback, scan callback, insert callback, or vacuum behavior is included
in this checkpoint.

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

The focused test command passed 31 selected tests:

- 15 `ec_spire::storage` unit tests
- 14 `ec_spire::meta` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Is `DataPageChain` the right stand-in for the first relation-page storage
   scaffold, or should this wait until real buffer/page writes land?
2. Should stale placements be unreadable here, or should the store only check
   locator integrity and let scan/consistency code decide state semantics?
3. Does returning a `SpirePlacementEntry` from the object write give the next
   root/control publish slice the right integration point?
