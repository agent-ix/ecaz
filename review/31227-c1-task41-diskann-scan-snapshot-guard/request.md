# Review Request: Task 41 DiskANN scan snapshot guard

## Summary

Task 41 follow-up for DiskANN scan snapshot ownership in
`src/am/ec_diskann/scan_state.rs`.

This slice adds a reusable `RegisteredSnapshotGuard` in
`src/storage/snapshot_guard.rs` and uses it for `ResolvedScanSnapshot` when a
DiskANN scan has to register a latest snapshot itself. Borrowed snapshots remain
borrowed.

Code commit: `d3ef03b5`

## Safety Effect

- Moves DiskANN-local registered snapshot unregister ownership into the shared
  `RegisteredSnapshotGuard`.
- Removes the DiskANN-specific `Drop` implementation that manually called
  `UnregisterSnapshot`.
- Preserves borrowed `xs_snapshot` and active snapshot behavior.
- Updates the unsafe comment baseline from `4078` to `4077`.

## Review Focus

- Confirm `RegisteredSnapshotGuard::latest` owns exactly the
  `RegisterSnapshot(GetLatestSnapshot())` / `UnregisterSnapshot` pair.
- Confirm `ResolvedScanSnapshot::borrowed` does not unregister executor-owned
  or active snapshots.
- Confirm the owned guard lives as long as the resolved snapshot object used by
  rerank heap fetches.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
