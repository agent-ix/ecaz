# Review Request: Task 41 Shared Index Scan Guard

## Summary

This slice promotes the module-local `index_beginscan` / `index_endscan`
RAII wrappers into a shared storage guard and migrates the two existing local
copies.

Code commit: `0a332ef5b6f15a2acc74e29b55f45bc565aaafa3`

## Changes

- Added `src/storage/scan_guard.rs` with shared
  `scan_guard::IndexScanGuard`.
- Exported the new storage guard module from `src/storage/mod.rs`.
- Removed the SPIRE custom-scan planner-local `IndexScanGuard`.
- Removed the HNSW debug-local `DebugIndexScanGuard`.
- Updated both callsites to construct the guard from typed
  `HeapRelationGuard`, `IndexRelationGuard`, and `ActiveSnapshotGuard`
  references, plus explicit `nkeys` / `norderbys`.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4237`
- After: `4235`

The reduction comes from deleting two module-local scan wrappers and keeping a
single shared unsafe boundary for `index_beginscan` / `index_endscan`.

## Review Focus

- Confirm `IndexScanGuard::begin` correctly preserves the PG18 and pre-PG18
  `index_beginscan` signatures.
- Confirm accepting typed guard references is the right shape for this shared
  constructor; it forces callers to have live relation and snapshot guards at
  construction time.
- Confirm drop ordering remains correct at migrated callsites:
  `DebugHeapBackedScan` stores `scan` before snapshot/relation guards, so the
  scan ends before its resources drop.
- Confirm the SPIRE SQL-placement scan behavior remains unchanged:
  `nkeys = 1`, `norderbys = 0`, followed by `index_rescan` and
  `index_getnext_slot`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
