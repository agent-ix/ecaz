# Task 41 LWLock storage guard

## Summary

This packet requests review for a Task 41 invariant #3 follow-up to packet 120.
It promotes the module-local HNSW concurrent DSM lock guard into the shared
storage guard family.

Code commit: `d4fbf0b0952cbb15f3faa92cab75633991c38d69`

Changes:

- Added `src/storage/lock_guard.rs` with shared `LwLockGuard`.
- Added production constructors:
  - `LwLockGuard::acquire_shared`
  - `LwLockGuard::acquire_exclusive`
- Added `from_acquired` and a narrow `from_acquired_with_release` adoption
  constructor so callback surfaces with injected test locks can still use the
  same RAII shape.
- Migrated `EcHnswConcurrentDsmLockOps` to return `LwLockGuard`.
- Removed the module-local `EcHnswConcurrentDsmLockGuard` and local
  `concurrent_dsm_lwlock_*` raw wrappers from HNSW parallel build.
- Updated `docs/ffi-inventory.md` line numbers and the unsafe baseline.

## Safety Effect

Raw `LWLockAcquire` / `LWLockRelease` calls are now centralized in
`src/storage/lock_guard.rs`. HNSW concurrent DSM graph code still supports
test lock injection, but production lock acquisition now goes through the
shared storage-level `LwLockGuard`.

Unsafe baseline moved from `3690` in packet 121 to `3687`, and
`src/am/ec_hnsw/build_parallel.rs` moved from `204` before packet 120 to `201`.

## Review Focus

- Confirm the shared guard is the right storage-family location and name for
  the Task 41 `LwLockGuard` primitive.
- Confirm HNSW test lock injection still avoids calling PostgreSQL LWLock APIs
  while using the same guard/drop shape.
- Confirm the raw LWLock scan is now limited to `src/storage/lock_guard.rs`.

## Validation

See `artifacts/manifest.md` for the command log and key result lines.
