---
id: 30169
title: SPIRE Object Manifest Codec
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 3f45514e
---
# Review Request: SPIRE Object Manifest Codec

## Summary

This checkpoint adds the per-epoch object manifest directory that maps PID to
object version and placement-entry locator. It complements the placement
directory codec and keeps the Phase 0 manifest shape concrete before root/control
publication wiring lands.

The checkpoint:

- adds `SpireObjectManifest`
- stores a deterministic sequence of `SpireManifestEntry` rows for one epoch
- sorts entries by PID during construction
- enforces one manifest entry per PID within an epoch
- rejects entry epochs that do not match the object manifest epoch
- adds a fixed object-manifest header with magic, metadata format version,
  reserved bytes, epoch, and entry count
- adds binary lookup by PID
- adds tests for round trip, sorting, PID lookup, duplicate rejection, epoch
  mismatch rejection, corrupt header rejection, and length mismatch rejection

No root/control page write, active-epoch publication, failed-publish cleanup,
relation buffer integration, build path, scan path, or vacuum behavior is
included in this checkpoint.

## Files To Review

- `src/am/ec_spire/meta.rs`

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 34 selected tests:

- 17 `ec_spire::meta` unit tests
- 15 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Is a sorted per-epoch object manifest the right bridge between active epoch
   selection and placement lookup?
2. Should the object manifest and placement directory remain separate blobs, or
   should the first relation-backed root/control page combine them?
3. Are the one-entry-per-PID invariants correct while replicas remain deferred?
