---
id: 30218
title: SPIRE Root Routing Draft Object
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: acefdd1a
---

# Review Request: SPIRE Root Routing Draft Object

## Summary

This checkpoint connects the routing object codec to the in-memory partitioned
single-level build draft.

- Adds `root_placement_tid` to `SpirePartitionedSingleLevelBuildInput`.
- Allocates the root PID before leaf PIDs.
- Builds a `SpireRoutingPartitionObject::root` from the route map.
- Sets each leaf object's `parent_pid` to the root PID.
- Publishes object and placement manifests containing the root object plus all
  centroid leaf objects.
- Keeps allocator cursor commits gated on published snapshot validation.

## Non-Goals

- No PostgreSQL AM callback persistence wiring.
- No root/control relation write path.
- No scan routing or nprobe/probe-width behavior.
- No remote placement, replica, or degraded-mode behavior changes.

## Review Focus

- Whether root PID allocation should precede leaf PID allocation for Phase 1.
- Whether leaf objects should point at the root PID immediately, or stay parent
  zero until multi-level routing lands.
- Whether including the root object in the same object and placement manifests
  is the right publication boundary for single-level SPIRE.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 121 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
