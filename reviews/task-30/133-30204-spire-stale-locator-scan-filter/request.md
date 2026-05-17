---
id: 30204
title: SPIRE Stale Locator Scan Filter
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 8ab7d3d8
---

# Review Request: SPIRE Stale Locator Scan Filter

## Summary

This checkpoint makes stored heap-locator staleness visible in scan semantics:
rows marked with `SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR` are no longer returned by
the visible-primary row overlay.

- Adds `SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR` to the blocked output flags used by
  `collect_snapshot_visible_primary_rows`.
- Keeps stale-locator rows encodable so future HOT/update repair and vacuum
  cleanup paths can mark rows without changing the row format.
- Extends the non-output scan fixture with a primary row carrying the stale
  locator flag.

## Non-Goals

- No HOT-chain repair implementation.
- No vacuum cleanup implementation.
- No heap visibility recheck.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 103 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 25 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
