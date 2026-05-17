# Review Request: Task 41 SPIRE Scan Heap Relation Guard

## Summary

This slice replaces SPIRE scan rerank's local owned/borrowed heap relation
drop logic with the shared `HeapRelationGuard` for owned heap relations.

Code commit: `7fa07dfbf2172a24c214aae7355f97e14cf74a5b`

## Changes

Updated `src/am/ec_spire/scan/relation.rs`:

- `ResolvedScanHeapRelation` now stores an optional
  `HeapRelationGuard` for owned heap relations.
- Borrowed scan-owned heap relations still remain raw, non-owned pointers.
- Removed the local `Drop` implementation that manually called
  `table_close`.
- Replaced direct `table_open` with `HeapRelationGuard::try_access_share`.
- Updated `scripts/unsafe_comment_baseline.txt` for line-number movement.

## Baseline

- Before: `4140`
- After: `4140`

The baseline count is unchanged because the removed raw open/close sites were
already documented, but the ownership is now centralized in the shared guard.

## Review Focus

- Confirm owned heap relations are now closed by `HeapRelationGuard` drop.
- Confirm borrowed scan heap relations are not closed by this wrapper.
- Confirm `relation` is derived from the guard before the guard is stored, so
  `as_ptr()` remains valid for the wrapper lifetime.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
