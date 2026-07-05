# Review Request: Task 41 HNSW parallel build snapshot guard

## Summary

Task 41 invariant #3 slice for HNSW parallel-build transaction snapshot
ownership.

This adds `RegisteredSnapshotGuard::transaction()` and uses it in
`src/am/ec_hnsw/build_parallel.rs`.

Code commit: `8c6b8c7e`

## Safety Effect

- Moves HNSW parallel build concurrent snapshot registration/unregistration
  into `RegisteredSnapshotGuard`.
- Removes the manual `unregister_snapshot` flag and raw `UnregisterSnapshot`
  calls from `EcHnswParallelBuildLeader`.
- Drops the snapshot guard before destroying the parallel context, preserving
  the old release ordering.
- Updates the unsafe comment baseline from `3701` to `3698`.

## Review Focus

- Confirm the registered transaction snapshot remains live through
  `table_parallelscan_initialize` and worker completion.
- Confirm the guard drops before `DestroyParallelContext` / `ExitParallelMode`,
  matching the old explicit unregister ordering.
- Confirm failure before leader construction still releases the registered
  snapshot by local guard drop.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
