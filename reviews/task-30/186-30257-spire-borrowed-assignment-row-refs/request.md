---
id: 30257
title: SPIRE Borrowed Assignment Row Refs
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: ece9d13b
---

# Review Request: SPIRE Borrowed Assignment Row Refs

## Summary

This checkpoint starts the A2/S6 hot-path cleanup without switching build or
scan to V2 leaves yet.

- Adds `SpireVecIdRef<'a>` for validated borrowed vector IDs.
- Adds `SpireLeafAssignmentRowRef<'a>` and `decode_prefix_ref()` so row-encoded
  V1/delta bytes can be inspected without allocating payload `Vec`s.
- Keeps owned `SpireLeafAssignmentRow::decode_prefix()` by converting from the
  borrowed view, preserving current callers.
- Moves primary-visible and delete-delta flag predicates into `storage.rs` so
  row-encoded and future V2 column paths share one visibility contract.
- Updates scan helpers to import the shared visibility helpers instead of
  owning duplicate flag logic.
- Updates Task 30 to record that borrowed V1 row refs and shared visibility
  have landed, while V2 column views and batch scorer APIs remain open.

## Non-Goals

- Does not switch scan collection to V2 column views.
- Does not add batch assignment scorer APIs yet.
- Does not remove the current owned V1 row decode path.
- Does not change candidate dedupe or top-k sorting behavior.

## Review Focus

- Whether the borrowed ref API is the right compatibility bridge for row-encoded
  deltas and tests.
- Whether visibility predicates belong in `storage.rs` or should move to a
  smaller assignment/row module before scan V2 migration.
- Whether preserving owned decode through borrowed decode keeps current callers
  stable enough for incremental migration.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 169 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
