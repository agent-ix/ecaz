---
id: 30210
title: SPIRE Visible Vec ID Uniqueness
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 41b8caae
---

# Review Request: SPIRE Visible Vec ID Uniqueness

## Summary

This checkpoint makes the scan-visible row collector reject duplicate visible
primary `vec_id` assignments.

The checkpoint:

- keeps Phase 1 boundary replicas, tombstones, delta deletes, and stale
  locators out of the visible primary row set
- adds a visible-row uniqueness guard after leaf and delta overlays are applied
- returns an explicit error if two visible primary rows expose the same
  `vec_id`
- adds a regression case where two leaf PIDs contain the same primary
  `vec_id`

This protects the Phase 1 per-index live `vec_id` uniqueness contract at the
scan helper boundary. Boundary-replica dedup remains future work because
boundary replicas are not visible primary rows in the Phase 1 collector.

## Files To Review

- `src/am/ec_spire/scan.rs`

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 106 selected tests:

- `ec_spire::assign`
- `ec_spire::build`
- `ec_spire::meta`
- `ec_spire::scan`
- `ec_spire::storage`
- `ec_spire::update`
- 2 pg catalog registration tests

## Reviewer Focus

1. Should duplicate visible primary `vec_id`s fail at the scan collector, or
   should this move earlier into epoch publish validation once persistence is
   wired?
2. Is it correct for Phase 1 to continue filtering boundary replicas instead
   of implementing boundary-replica dedup now?
