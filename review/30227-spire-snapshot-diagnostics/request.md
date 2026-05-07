---
id: 30227
title: SPIRE Snapshot Diagnostics
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: ee7f6341
---

# Review Request: SPIRE Snapshot Diagnostics

## Summary

This checkpoint adds the first SPIRE admin/diagnostics helper over published
partition-object snapshots.

- Adds `src/am/ec_spire/diagnostics.rs`.
- Adds `SpireSnapshotDiagnostics` with epoch, consistency mode, object and
  placement counts, local-store count, placement-state counts, object-kind
  counts, routing-child count, assignment counts, and available object bytes.
- Reads object contents only for available local placements.
- Counts degraded unavailable/skipped placements without trying to read their
  objects.
- Keeps diagnostics as an internal helper; no SQL/admin surface is exposed yet.

## Non-Goals

- No SQL function or extension-visible diagnostics view.
- No relation-backed persistence wiring.
- No remote placement reads.
- No recall/latency measurement.

## Review Focus

- Whether diagnostics should count local stores across unavailable placements
  as this helper does, or only across readable placements.
- Whether unavailable object kind/cardinality should remain unknown until
  replicated manifests carry kind/cardinality metadata.
- Whether `available_object_bytes` is enough for the first status surface, or
  if manifest and placement bytes should be split immediately.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 136 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
