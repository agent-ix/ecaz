# Review Request: Cost Epoch Snapshot Cache

Task: 142 — SPIRE Epoch-Keyed Caching
Branch: `task-142-spire-epoch-cache-overhead`
Code commit: `4efd13e9acb3f5018188b0f3d931ecaf34ffa6d5`

## Summary

This slice adds a backend-local planner cost snapshot cache keyed by
`(index_oid, active_epoch)`.

The cache stores the derived cost inputs that were previously rebuilt by
planner callbacks:

- `SpireActiveSnapshotDiagnostics`
- `SpireIndexHierarchySnapshot`

`compute_amcostestimate` and the PG18 `amgettreeheight` callback now share that
epoch-stable snapshot. Cache hits avoid repeating the active snapshot
diagnostics walk and hierarchy snapshot walk. Diagnostic SQL paths still use
their existing uncached snapshot functions.

## Scope Notes

- This advances Task 142 Phase 1 cost-callback caching.
- It does not yet add publish-time stats to the on-disk epoch manifest; that is
  left for a follow-up because the epoch manifest is fixed-width and constructed
  across build/update/insert/vacuum publish paths.
- This does not change the cost model equations or relation/session option
  resolution.

## Validation

Packet-local logs:

- `artifacts/cargo-test-cost-epoch-snapshot-cache.log`
  - `cost_epoch_snapshot_cache_reuses_epoch_snapshot ... ok`
- `artifacts/cargo-test-cost-module-regression.log`
  - `am::ec_spire::cost::tests` ran 8 tests, all passed

`git diff --check` passed locally.
