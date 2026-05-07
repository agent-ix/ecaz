---
id: 30183
title: SPIRE Delta Partition Object Codec
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 236705a3
---

# Review Request: SPIRE Delta Partition Object Codec

## Summary

This checkpoint adds the SPIRE delta partition-object codec chosen by the
Phase 0 insert/delete lifecycle.

- Adds `SpireDeltaPartitionObject`.
- Encodes/decodes delta objects with `SpirePartitionObjectKind::Delta`.
- Requires delta objects to reference a non-zero base leaf PID through
  `parent_pid`.
- Requires no child PIDs in Phase 1 delta objects.
- Requires each row to set exactly one of:
  - `SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT`
  - `SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE`
- Requires delete-delta rows to be tombstoned.
- Rejects tombstoned insert-delta rows.
- Rejects boundary-replica delta rows in Phase 1.

## Non-Goals

- No local object-store write/read support for delta objects yet.
- No insert/delete AM callback behavior.
- No relation-backed persistence.
- No delta compaction or cleanup execution.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 76 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 18 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
