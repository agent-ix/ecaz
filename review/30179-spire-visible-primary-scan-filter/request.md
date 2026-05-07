---
id: 30179
title: SPIRE Visible Primary Scan Filter
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 02c0f3c2
---

# Review Request: SPIRE Visible Primary Scan Filter

## Summary

This checkpoint adds a pure scan-side visible-primary filter on top of the
snapshot leaf row collector.

- Adds `collect_snapshot_visible_primary_rows`.
- Keeps rows with `SPIRE_ASSIGNMENT_FLAG_PRIMARY`.
- Filters out:
  - boundary-replica rows
  - tombstone rows
  - delete-delta rows
  - non-primary rows
- Preserves PID, object version, row index, and assignment context for rows that
  remain visible.

## Non-Goals

- No `amgettuple` behavior change.
- No distance scoring, ordering, or heap visibility checks.
- No relation-backed object persistence.
- No delete/vacuum mutation path.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 68 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 6 `ec_spire::build` unit tests
  - 30 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
