# Review Request: SPIRE Scan Leaf/Delta Read Grouping

- Branch: `task30-spire-partition-object-spec`
- Code commit: `3cc56716d93921f3f2d5b935225072b7ae114be8`
- Scope: Phase 4 scan read-plan grouping for selected leaves and deltas

## Summary

This checkpoint extends the scan grouping boundary from selected leaf routes to
the leaf/delta read plan shape needed for future store-local fetch.

It:

- adds `SpireDeltaObjectRoute` and `SpireStoreObjectReadGroup`;
- groups selected leaf routes and matching delta routes by each object's own
  `(node_id, local_store_id)`;
- filters delta routes whose parent leaf was not selected;
- keeps route order stable inside each store group and store order
  deterministic;
- keeps current live candidate collection behavior unchanged.

This does not yet discover delta headers through the grouping plan or read from
auxiliary store relations. It is the pure planning boundary that the live
delta-header discovery and multi-store object readers can consume next.

## Files

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

- Whether deltas should be grouped by their own placement store rather than the
  selected parent leaf's store. The current helper groups by the delta object's
  placement to keep the read plan honest if a future degraded/migration state
  temporarily separates them.
- Whether filtering non-selected parent leaves belongs at this layer.
- Whether a later live implementation should discover delta headers once per
  scan and then feed this grouping helper, instead of scanning the manifest
  separately per leaf as the current candidate path still does.

## Validation

- `cargo test group_leaf_and_delta_reads_by_local_store --lib`
- `cargo test group_leaf_routes_by_local_store --lib`
- `cargo test collect_quantized_routed_probe_candidates --lib`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
