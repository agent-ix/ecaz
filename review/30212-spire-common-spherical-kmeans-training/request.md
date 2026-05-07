---
id: 30212
title: SPIRE Common Spherical K-Means Training
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 3e9fb973
---

# Review Request: SPIRE Common Spherical K-Means Training

## Summary

This checkpoint starts the Phase 0 reuse plan by moving IVF-neutral spherical
k-means helpers into `src/am/common/training.rs`.

The checkpoint:

- adds common `SphericalKMeansModel`
- adds common auto-`nlists`, deterministic sampling, vector normalization,
  spherical k-means training, and centroid assignment helpers
- keeps AM-specific error labels so `ec_ivf` keeps its existing error strings
- turns `src/am/ec_ivf/training.rs` into wrappers around the common helpers
- keeps the grouped-PQ training helpers in the same common module

This does not change `ec_ivf` page formats, metadata, posting-list mutation, or
SPIRE build callbacks. It only creates the shared centroid-training boundary
that SPIRE can use without importing `ec_ivf` private modules.

## Files To Review

- `src/am/common/training.rs`
- `src/am/ec_ivf/training.rs`

## Validation

- `cargo fmt`
- `cargo test --lib training --no-default-features --features pg18`
- `cargo test --lib ec_spire --no-default-features --features pg18`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused training command passed 15 selected tests, covering common grouped
PQ training, `ec_ivf` spherical k-means wrappers, and `ec_ivf` build training
sampling. The focused `ec_spire` command passed 107 selected tests.

## Reviewer Focus

1. Is `src/am/common/training.rs` the right boundary for SPIRE to reuse
   spherical k-means training?
2. Are AM-specific error labels sufficient, or should the common helpers use a
   typed error context instead?
3. Does keeping `ec_ivf` as wrappers preserve compatibility cleanly enough for
   this first factoring step?
