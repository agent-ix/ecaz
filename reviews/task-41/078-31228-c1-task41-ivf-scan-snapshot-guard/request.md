# Review Request: Task 41 IVF scan snapshot guard

## Summary

Task 41 follow-up for IVF scan snapshot ownership in
`src/am/ec_ivf/scan.rs`.

This slice reuses the shared `RegisteredSnapshotGuard` introduced for DiskANN
and applies it to `ResolvedIvfScanSnapshot` when an IVF scan must register a
latest snapshot itself. Borrowed `xs_snapshot` and active snapshots remain
borrowed.

Code commit: `6ac20bdb`

## Safety Effect

- Moves IVF-local registered snapshot unregister ownership into
  `RegisteredSnapshotGuard`.
- Removes the IVF-specific `Drop` implementation that manually called
  `UnregisterSnapshot`.
- Preserves borrowed `xs_snapshot` and active snapshot behavior.
- Updates the unsafe comment baseline from `4077` to `4076`.

## Review Focus

- Confirm `ResolvedIvfScanSnapshot::owned` stores the guard, not just the raw
  snapshot pointer.
- Confirm borrowed executor-owned and active snapshots do not unregister on
  scan cleanup.
- Confirm the guard-backed snapshot remains live for heap rerank fetches.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
