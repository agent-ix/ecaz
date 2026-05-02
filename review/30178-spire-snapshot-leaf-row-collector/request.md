---
id: 30178
title: SPIRE Snapshot Leaf Row Collector
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 5bfd42e8
---

# Review Request: SPIRE Snapshot Leaf Row Collector

## Summary

This checkpoint adds a pure scan-side collector over a validated SPIRE
published epoch snapshot and the local object-store abstraction.

- Adds `SpireLeafScanRow`.
- Adds `collect_snapshot_leaf_rows`.
- Revalidates the supplied `SpirePublishedEpochSnapshot`.
- Resolves object manifest entries through the placement directory.
- Reads available local leaf objects from `SpireLocalObjectStore`.
- Emits leaf assignment rows with PID, object version, and row index context.
- Skips `Unavailable` and `Skipped` placements in degraded mode.

## Non-Goals

- No `ambeginscan`, `amrescan`, `amgettuple`, or `amendscan` behavior change.
- No distance scoring, ordering, or LIMIT handling.
- No relation-backed object persistence.
- No remote object reads.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 67 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 6 `ec_spire::build` unit tests
  - 30 `ec_spire::meta` unit tests
  - 2 `ec_spire::scan` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
