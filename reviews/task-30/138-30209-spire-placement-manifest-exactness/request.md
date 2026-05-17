---
id: 30209
title: SPIRE Placement Manifest Exactness
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 599ead33
---

# Review Request: SPIRE Placement Manifest Exactness

## Summary

This checkpoint tightens the published SPIRE epoch snapshot contract so the
object manifest and placement directory expose the same PID set.

The checkpoint:

- keeps the existing requirement that every object-manifest PID resolves to a
  placement entry
- adds the inverse requirement that every placement-directory PID appears in
  the object manifest
- rejects published snapshots with orphan placement entries instead of letting
  inactive object bytes become part of an active epoch's placement metadata
- adds a regression case covering an extra placement PID beside a valid
  manifest entry

This is still metadata/draft validation only; it does not wire PostgreSQL AM
callback persistence.

## Files To Review

- `src/am/ec_spire/meta.rs`

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 105 selected tests:

- `ec_spire::assign`
- `ec_spire::build`
- `ec_spire::meta`
- `ec_spire::scan`
- `ec_spire::storage`
- `ec_spire::update`
- 2 pg catalog registration tests

## Reviewer Focus

1. Should a published epoch require exact object-manifest/placement-directory
   PID equality, or should orphan placements be tolerated for diagnostics?
2. Does this invariant match the Phase 1 single-store assumption that replicas
   are deferred and each live PID has one placement entry?
