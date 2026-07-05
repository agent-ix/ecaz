---
id: 30217
title: SPIRE Routing Object Codec
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: b7990e38
---

# Review Request: SPIRE Routing Object Codec

## Summary

This checkpoint adds storage-codec support for SPIRE root/internal routing
partition objects without wiring them into PostgreSQL AM callbacks.

- Adds `SpireRoutingChildEntry` with centroid index, child PID, and centroid
  vector.
- Adds `SpireRoutingPartitionObject` for `Root` and `Internal` object kinds.
- Encodes routing objects as the existing partition-object header plus
  dimensions, reserved bytes, and fixed-width child entries with centroid
  vectors.
- Validates parent/child PID semantics:
  - root routing objects require `parent_pid = 0`;
  - internal routing objects require non-zero `parent_pid`;
  - routing objects require `level > 0`;
  - child PIDs must be non-zero and centroid indexes must be dense/in-order.
- Adds local object-store insert/read support for routing objects and keeps
  header dispatch working across leaf, delta, and root objects.

## Non-Goals

- No root object allocation in the partitioned build draft yet.
- No manifest publication of routing objects yet.
- No persistent relation-backed AM callback writes.
- No remote route map, replica, or degraded-mode behavior changes.

## Review Focus

- Whether the routing object body shape is sufficient for the single-level IVF
  foundation and future multi-level extension.
- Whether dense/in-order centroid indexes should be enforced at the codec layer
  or only at route-map/build layers.
- Whether root/internal parent/level validation is too strict for empty-index or
  future remote cases.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 121 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
