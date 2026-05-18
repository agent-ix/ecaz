---
id: 30202
title: SPIRE Delta Assignment Role Flags
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 3b0e35b2
---

# Review Request: SPIRE Delta Assignment Role Flags

## Summary

This checkpoint tightens delta partition-object row roles.

- Requires delta-insert rows to also be primary assignment rows.
- Rejects delta-delete rows that also carry the primary flag.
- Extends invalid delta-flag coverage for missing-primary inserts and primary
  delete tombstones.

## Non-Goals

- No payload-format policy changes for delete rows.
- No scan overlay logic changes.
- No delta merge/compaction.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 102 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 24 `ec_spire::storage` unit tests
  - 17 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
