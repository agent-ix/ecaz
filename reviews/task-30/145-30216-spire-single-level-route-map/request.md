---
id: 30216
title: SPIRE Single-Level Route Map
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 1004b3d4
---

# Review Request: SPIRE Single-Level Route Map

## Summary

This checkpoint makes the Phase 1 SPIRE centroid-to-leaf-PID mapping explicit
in the in-memory partitioned build draft.

- Adds `SpireSingleLevelRouteEntry` and `SpireSingleLevelRouteMap`.
- Builds a route-map entry for every centroid with the centroid index, centroid
  vector, and allocated leaf PID.
- Validates route-map dimensions, centroid ordering, finite centroid components,
  non-zero child PIDs, and centroid/PID count agreement.
- Adds `route_pid_for_vector` so future scan/build callers have a concrete
  query-vector-to-leaf-PID contract using the shared spherical k-means centroid
  assignment helper.
- Stores the route map inside `SpirePartitionedSingleLevelBuildDraft` next to
  the object and placement manifests.

## Non-Goals

- No persistent root/internal routing object yet.
- No PostgreSQL `ambuild` callback wiring.
- No query scan execution or probe-width/nprobe behavior.
- No remote route map or replica behavior.

## Review Focus

- Whether the route map belongs in the build draft before persistent root object
  encoding lands.
- Whether the route map should keep full centroid vectors or only centroid
  indexes plus PID references at this layer.
- Whether `route_pid_for_vector` should continue to reuse the common spherical
  k-means assignment helper or factor a lower-allocation routing helper before
  scan wiring.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 115 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
