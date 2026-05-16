# SPIRE Local Correctness Matrix

## Scope

This packet records the Phase 8 local correctness matrix for SPIRE PG18
coverage.

Code checkpoint: `a1665209` (`Add SPIRE maintenance scheduler packet`)

## Changes

- Adds `artifacts/local-correctness-matrix.md`.
- Groups representative focused PG18 filters by behavior lane:
  - build/scan
  - recursive/top-graph routing
  - insert/update epochs
  - VACUUM/delete visibility
  - maintenance publish safety
  - storage and old-epoch debt
  - placement/replica diagnostics
  - remote contract drift guards
  - distributed manifest contracts
  - planner/operator surfaces
- Marks the Phase 8 local correctness matrix item complete.

## Files

- `plan/tasks/30-spire-ivf-foundation.md`
- `review/30626-spire-local-correctness-matrix/artifacts/local-correctness-matrix.md`

## Validation

- `rg -n "fn test_ec_spire_.*(scan|vacuum|wal|maintenance|epoch|storage|recursive|remote_search|insert|delete|update|split|merge|sanity|cost|planner|top_graph|allocator)" src/lib.rs`
- `rg -n "fn test_ec_spire_" src/lib.rs | wc -l` reported `110`.
- `git diff --check`

## Notes

No tests were run for this packet. It is a coverage map and task-tracking
checkpoint built from the existing test inventory.
