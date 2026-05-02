---
id: 30211
title: SPIRE Delta Degraded Base Placement Guard
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 068ef325
---

# Review Request: SPIRE Delta Degraded Base Placement Guard

## Summary

This checkpoint makes delta epoch draft construction fail early when the base
snapshot contains non-available placements.

The checkpoint:

- validates that every base snapshot placement is `Available` before carrying
  entries into a new delta epoch
- returns a targeted error for degraded base snapshots with `Skipped` or
  `Unavailable` placements
- keeps local single-store update publication fail-closed until the skipped
  placement can be repaired or a later degraded-write policy is designed
- adds a regression case for a degraded base snapshot with a skipped placement

This does not remove degraded scan support. It only prevents the Phase 1 delta
publisher from building a replacement epoch from incomplete base object bytes.

## Files To Review

- `src/am/ec_spire/update.rs`

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 107 selected tests:

- `ec_spire::assign`
- `ec_spire::build`
- `ec_spire::meta`
- `ec_spire::scan`
- `ec_spire::storage`
- `ec_spire::update`
- 2 pg catalog registration tests

## Reviewer Focus

1. Is fail-closed delta publication from degraded base snapshots the right
   Phase 1 behavior?
2. Should future local multi-store degraded writes require a separate policy
   rather than reusing this single-store delta draft path?
