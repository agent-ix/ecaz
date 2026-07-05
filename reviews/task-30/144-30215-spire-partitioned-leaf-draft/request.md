---
id: 30215
title: SPIRE Partitioned Leaf Draft
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 1b625a9d
---

# Review Request: SPIRE Partitioned Leaf Draft

## Summary

This checkpoint starts the SPIRE build path past the monolithic single-leaf
draft by adding a per-centroid partitioned leaf epoch draft.

- Adds `SpirePartitionedSingleLevelBuildInput` and
  `SpirePartitionedSingleLevelBuildDraft`.
- Validates centroid plan shape before using the plan for PID/object fanout.
- Allocates one leaf PID per centroid, including empty centroids, so the
  manifest and placement map cover the full centroid route set.
- Groups assignment inputs by centroid assignment index and writes one
  `SpireLeafPartitionObject` per centroid into the local object store.
- Builds object and placement manifests for the full centroid PID set, validates
  the published snapshot, and only commits PID/local vec-id allocator cursors
  after validation succeeds.
- Builds all leaf objects before inserting them into the object store, preventing
  partial local-store writes when a later centroid group has invalid assignment
  input.

## Non-Goals

- No PostgreSQL AM callback persistence wiring yet.
- No root/internal routing object persistence yet.
- No scan routing by query-to-centroid distance yet.
- No remote placement, replica, or degraded-mode implementation changes.
- The older monolithic single-leaf draft remains available for the earlier
  foundation tests.

## Review Focus

- Whether the draft boundary is the right next step before live `ambuild`
  relation persistence.
- Whether one PID per centroid, including empty centroids, is the right Phase 1
  route-manifest contract.
- Whether the all-leaf-build-before-object-store-insert behavior is sufficient
  for this in-memory draft layer.
- Whether the duplicated manifest/root-control bundle methods should be factored
  now or left until the monolithic draft is removed.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 113 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
