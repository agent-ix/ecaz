# Review Request: SPIRE Scan Leaf Route Store Grouping

- Branch: `task30-spire-partition-object-spec`
- Code commit: `2224411ca182ee2fd5cb12611692d655402f3db6`
- Scope: Phase 4 scan grouping boundary for local multi-store fetch

## Summary

This checkpoint adds the first scan-side grouping boundary for future
multi-store local fetch.

It:

- adds `SpireStoreLeafRouteGroup`;
- groups selected quantized scan leaf routes by `(node_id, local_store_id)`;
- preserves route order inside each store group while processing stores in
  deterministic key order;
- feeds the grouped routes through the existing synchronous object-reader and
  candidate-scoring flow;
- leaves global candidate ranking and reranking unchanged.

This does not open auxiliary store relations, perform parallel reads, or make
any multi-NVMe performance claim. It only makes the scan path's selected PID to
store-group boundary explicit.

## Files

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

- Whether grouping selected leaf routes before leaf/delta reads is the right
  first scan boundary for the Phase 4 store-grouped fetch design.
- Whether deterministic store-key order plus per-store route-order preservation
  is appropriate before true parallel fetch.
- Whether the grouping should later include delta placements directly rather
  than continuing to discover deltas per routed leaf PID.

## Validation

- `cargo test group_leaf_routes_by_local_store --lib`
- `cargo test collect_quantized_routed_probe_candidates --lib`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
