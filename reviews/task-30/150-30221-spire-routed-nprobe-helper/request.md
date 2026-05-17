---
id: 30221
title: SPIRE Routed nprobe Helper
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 30e035a4
---

# Review Request: SPIRE Routed nprobe Helper

## Summary

This checkpoint extends the root-routed scan helper from one nearest centroid
route to top-`nprobe` leaf routes.

- Adds `collect_snapshot_routed_probe_leaf_rows`.
- Sorts root routing children by query/centroid inner-product score with a
  stable centroid-index tie break.
- Reads rows from each selected leaf PID through the same parent-PID and
  placement-state checks as the single-route helper.
- Keeps `collect_snapshot_routed_leaf_rows` as the `nprobe = 1` convenience
  path.
- Validates `nprobe > 0`, query dimensions, finite query components, and
  non-zero query vectors.

## Non-Goals

- No AM callback scan execution.
- No candidate scoring, dedup, or rerank path.
- No adaptive probe-width or planner/GUC surface.
- No remote placement or replica behavior.

## Review Focus

- Whether the helper should clamp oversized `nprobe` to child count as it does
  now, or reject oversized probe counts earlier.
- Whether scan routing should keep this local inner-product ranking or factor a
  shared top-centroid helper beside `am/common/training`.
- Whether degraded empty routed-leaf results need explicit diagnostics before
  callback wiring.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 126 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
