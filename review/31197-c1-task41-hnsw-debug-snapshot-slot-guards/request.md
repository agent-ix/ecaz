# Review Request: Task 41 HNSW Debug Snapshot/Slot Guards

## Summary

This slice migrates HNSW debug heap-backed scan helpers from local
snapshot/slot guard implementations to the shared storage guards introduced
in `31195`.

Code commit: `69807c26ebc1bf5b69cdd02575f9db024a9bc2eb`

## Changes

- Removed the local `DebugActiveSnapshotGuard` from
  `src/am/ec_hnsw/scan_debug.rs`.
- Removed the local `DebugTupleSlotGuard` from
  `src/am/ec_hnsw/scan_debug.rs`.
- Reused `storage::snapshot_guard::ActiveSnapshotGuard` and
  `storage::slot_guard::TupleTableSlotGuard` in HNSW debug heap-backed scan
  helpers.
- Added `ActiveSnapshotGuard::latest_after_command_counter()` behind
  `#[cfg(any(test, feature = "pg_test"))]` so pg-test debug helpers preserve
  the previous `CommandCounterIncrement` plus latest-snapshot behavior without
  exposing unused non-test API.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4238`
- After: `4237`

This removes one residual unsafe site by deleting the local slot guard and
moves the snapshot CCI preamble to a shared constructor with an explicit
SAFETY comment.

## Review Focus

- Confirm `latest_after_command_counter()` preserves the prior debug helper
  ordering: `CommandCounterIncrement`, `RegisterSnapshot(GetLatestSnapshot())`,
  `PushActiveSnapshot`, then drop-time `PopActiveSnapshot` and
  `UnregisterSnapshot`.
- Confirm `TupleTableSlotGuard::single_for_heap()` covers the former
  `DebugTupleSlotGuard` lifetime and drop behavior for
  `debug_profile_ordered_scan_with_heap_fetch`.
- Confirm the pg-test cfg on `latest_after_command_counter()` is the right API
  surface for this debug-only command-counter snapshot acquisition.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
