---
id: 30220
title: SPIRE Root-Routed Scan Helper
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 02da5996
---

# Review Request: SPIRE Root-Routed Scan Helper

## Summary

This checkpoint adds the first scan-side use of the published SPIRE root routing
object, still below live PostgreSQL scan callbacks.

- Adds `SpireRoutedLeafScanRows`.
- Adds `collect_snapshot_routed_leaf_rows`, which:
  - validates the published snapshot;
  - loads exactly one available root routing object;
  - routes a query vector to the nearest centroid child PID;
  - reads only the routed leaf object's assignment rows;
  - verifies the routed leaf object's parent PID matches the root PID.
- Preserves strict/degraded placement behavior:
  - strict mode errors on non-available placements;
  - degraded mode can skip an unavailable or skipped routed leaf and return no
    rows for that route;
  - stale placements remain rejected.

## Non-Goals

- No AM callback scan execution.
- No `nprobe`/multi-probe behavior yet; this is one nearest-centroid route.
- No candidate scoring or rerank path.
- No remote placement or replica behavior.

## Review Focus

- Whether root-object discovery should require exactly one available root at
  this helper boundary.
- Whether degraded mode should return an empty routed leaf result for unavailable
  target leaves or surface a degraded-route diagnostic immediately.
- Whether parent-PID validation belongs in the scan helper or should rely on
  published snapshot validation.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 124 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
