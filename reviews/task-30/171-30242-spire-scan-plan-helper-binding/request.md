---
id: 30242
title: SPIRE Scan Plan Helper Binding
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 9588fd38
---

# Review Request: SPIRE Scan Plan Helper Binding

## Summary

This checkpoint binds the resolved single-level SPIRE scan plan to the
helper-level routed scan path.

- Adds a scan helper that consumes `SpireSingleLevelScanPlan` and forwards its
  effective `nprobe`, assignment payload format, candidate limit, and rerank
  width to the routed quantized scoring and exact rerank helper.
- Preserves an explicit empty-plan path for `nprobe = 0`.
- Keeps live AM callback wiring and persistence untouched.

## Non-Goals

- No PostgreSQL callback behavior change.
- No persistence, WAL, relation storage, or remote placement work.
- No new scan cost model or planner integration.

## Review Focus

- Whether `SpireSingleLevelScanPlan` now has a clear consumption boundary in
  the scan helper layer.
- Whether the empty-plan behavior is acceptable for a relation with no visible
  leaf objects.
- Whether the exact-rerank callback shape remains appropriate before live heap
  TID lookup is wired.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 157 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
