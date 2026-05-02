---
id: 30214
title: SPIRE Single-Level Centroid Plan
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 09ae4009
---

# Review Request: SPIRE Single-Level Centroid Plan

## Summary

This checkpoint adds the first SPIRE-owned build-path helper that consumes the
new common spherical k-means boundary.

The checkpoint:

- adds `SpireSingleLevelCentroidPlan`
- adds `train_single_level_centroid_plan`
- resolves auto `nlists` through common training
- trains centroids through common spherical k-means with an `ec_spire` error
  label
- assigns each source vector to a centroid index
- adds tests for two-way routing, auto-list resolution, and bad vector
  rejection

This still does not wire `ambuild`, relation-backed persistence, quantizer
encoding, or leaf-object partition fanout. It is the build-path routing
foundation that will feed those later steps.

## Files To Review

- `src/am/ec_spire/build.rs`

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 109 selected tests:

- `ec_spire::assign`
- `ec_spire::build`
- `ec_spire::meta`
- `ec_spire::scan`
- `ec_spire::storage`
- `ec_spire::update`
- 2 pg catalog registration tests

## Reviewer Focus

1. Is a SPIRE-owned centroid plan the right boundary before adding partition
   object fanout?
2. Should the plan store assignment indexes as `u32`, or should it carry
   allocated PIDs immediately once the next build slice creates leaf objects
   per centroid?
