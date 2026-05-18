# Task 41 resource lint closeout follow-ups

## Summary

This packet requests review for the remaining Task 41 invariant #1/#3
deep-audit follow-ups from:

- `reviews/task-41/122-lwlock-storage-guard/feedback/2026-05-18-02-reviewer.md`
- `reviews/task-41/126-dylint-ffi-guard-lint/feedback/2026-05-18-02-reviewer.md`

Code commit: `d78d0485a0b89527c7737cd032e44c46feb8ca45`

Changes:

- Replaced SPIRE relation-store's local `relation_open`/`relation_close`
  wrapper with the shared `RelationGuard`.
- Expanded `scripts/ffi_lint.py` coverage for:
  - `table_open` / `table_close`
  - `relation_open` / `relation_close`
  - `PushActiveSnapshot` / `PopActiveSnapshot`
  - `index_beginscan` / `index_endscan`
  - `heap_beginscan` / `heap_endscan`
- Added negative and allowed fixtures for the new lint families.
- Added Task 41 closeout notes for FFI inventory freshness, `pg_finfo_*`
  exceptions, empty `ResourceOwner` surface, pgrx-managed SPI use, syntactic
  Dylint scope, and Task 38-dependent leak-smoke content.

## Safety Effect

The #3 raw-resource gate now covers the remaining wrapper families named in the
deep audit. A source sweep for the newly covered APIs reports only
`src/storage/*_guard.rs` callsites.

The #1 notes document the remaining scope decisions rather than changing
runtime behavior: inventory drift is already checked by `ffi_audit.py --check`,
`pg_finfo_*` is prefix-classified as metadata-only, and semantic Dylint
reachability remains future hardening scope.

## Review Focus

- Check that replacing the SPIRE-local relation wrapper with `RelationGuard`
  preserves the same open/close ownership and reverse drop order.
- Check the new `ffi_lint.py` rules and self-test fixtures.
- Check that the Task 41 closeout notes accurately reflect the deep-audit
  remaining items without crossing into invariant #2.

## Validation

See `artifacts/manifest.md`.
