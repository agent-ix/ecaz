# Task 50 Review Request: Locked Buffer Tuple Visitor

## Summary

This slice continues the P4 page tuple / line-pointer burndown by adding a safe
`LockedBufferGuard::visit_tuple_bytes` API. The API requires an existing locked
buffer guard, validates the requested line pointer and tuple bounds, and only
exposes the borrowed page bytes for the duration of a visitor closure.

Rollout in this packet:

- `src/am/ec_diskann/scan_state.rs`: removed the local raw page tuple copier
  used while materializing DiskANN data pages.
- `src/am/ec_diskann/routine.rs`: removed the immutable vacuum tuple-byte
  helper and validates expected bytes through the locked-buffer visitor before
  entering the remaining WAL/mutable tuple path.
- `src/storage/buffer_guard.rs`: centralized immutable locked-page tuple
  iteration behind the RAII buffer guard.

The direct `unsafe { ... }` count moved from `1218` to `1213`.

Code commit: `b534684272310df5dffe06cd5f54deb4d0835ce3`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- DiskANN tuple-helper scan shows the remaining `PageGetItem` boundary is now
  centralized in `src/storage/buffer_guard.rs`.
- Unsafe ledger check passed for `1213` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
