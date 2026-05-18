# Review Request: Task 41 IVF Vacuum Debug Index Guards

## Summary

This slice migrates the IVF pg-test vacuum debug helpers from manual
`index_open` / `index_close` pairs to `IndexRelationGuard`.

Code commit: `a4cd8438bd8b9955631de5a2e860635e06da7c04`

## Changes

Updated `src/am/ec_ivf/vacuum.rs`:

- `debug_ec_ivf_vacuum_stats` now uses
  `IndexRelationGuard::access_share`.
- `debug_ec_ivf_vacuum_remove_heap_tids` now uses
  `IndexRelationGuard::open` with the existing
  `ShareUpdateExclusiveLock`.
- `IndexVacuumInfo.index` is populated from `guard.as_ptr()` while the guard
  remains live across `ec_ivf_ambulkdelete` and `ec_ivf_amvacuumcleanup`.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4146`
- After: `4142`

## Review Focus

- Confirm the guard lock modes match the removed manual opens.
- Confirm the raw `IndexVacuumInfo.index` pointer remains covered by the live
  guard through both vacuum callbacks.
- Confirm copied `IndexBulkDeleteResult` data is returned only after callback
  work is complete, before the guard drops.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
