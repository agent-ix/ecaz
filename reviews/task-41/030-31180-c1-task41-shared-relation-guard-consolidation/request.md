# Review Request: Task 41 Shared Relation Guard Consolidation

Code commit: `a32237f11b18849526f7d5fb1e4d18bb7b80486d`

## Summary

This checkpoint addresses reviewer feedback from packets 31172 and 31173 by
consolidating PostgreSQL relation guards.

- Adds `src/storage/relation_guard.rs` with `IndexRelationGuard` and
  `HeapRelationGuard`.
- Removes the local `AccessShareIndexRelation` from `src/lib.rs`.
- Removes the local `AccessShareIndexRelation` and `AccessShareHeapRelation`
  from `src/am/ec_spire/dml_frontdoor/mod.rs`.
- Removes the pg_test-only `DebugIndexRelation` from
  `src/am/ec_spire/coordinator/debug.rs`.

## Safety Delta

- Baseline entries remain `4301` -> `4301`.
- This is consolidation rather than count burndown: the Drop-on-pgrx-error
  invariant and relation close ownership now live in one shared primitive.
- Existing AM-validation helpers in `src/lib.rs` remain as thin wrappers over
  the shared index guard.

## Reviewer Focus

- Confirm `IndexRelationGuard::open` and `try_open` cover the previous three
  index relation shapes: error-on-null, option-on-null, and dynamic lock mode.
- Confirm `HeapRelationGuard::try_access_share` preserves the DML frontdoor
  catalog heap relation null behavior.
- Confirm `coordinator/debug.rs` still preserves relation lifetime and lock mode
  across the pg_test debug helper closures.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see
`artifacts/manifest.md`.
