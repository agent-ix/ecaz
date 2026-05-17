# Review Request: Task 41 HNSW scan snapshot guard

## Summary

Task 41 follow-up for HNSW grouped heap-rerank snapshot ownership in
`src/am/ec_hnsw/scan.rs`.

This slice applies the shared `RegisteredSnapshotGuard` to
`ResolvedHnswScanSnapshot`, matching the DiskANN and IVF snapshot guard shape.
Borrowed executor-owned and active snapshots remain borrowed.

Code commit: `c5e2a488`

## Safety Effect

- Moves HNSW-local registered snapshot unregister ownership into
  `RegisteredSnapshotGuard`.
- Removes the HNSW-specific bool-owned `Drop` implementation that manually
  called `UnregisterSnapshot`.
- Preserves borrowed `xs_snapshot` and active snapshot behavior.
- Updates the unsafe comment baseline from `4076` to `4075`.

## Review Focus

- Confirm `ResolvedHnswScanSnapshot::owned` stores the guard, not just the raw
  snapshot pointer.
- Confirm borrowed executor-owned and active snapshots do not unregister on
  scan cleanup.
- Confirm `GroupedHeapRerankState` field order still drops the slot before the
  snapshot guard and heap relation guard.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
