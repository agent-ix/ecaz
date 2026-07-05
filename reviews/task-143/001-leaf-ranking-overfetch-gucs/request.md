# Review Request: Task 143 Packet 001 — Leaf Ranking / Overfetch GUCs

## Summary

This slice adds default-off Task 143 routing controls:

- `ec_spire.leaf_score_only_routing`: ranks final recursive leaf candidates by leaf score only, using accumulated parent path score only as a tie-breaker.
- `ec_spire.route_overfetch_multiplier`: widens recursive route exploration with ceiling arithmetic while keeping final scanned leaf routes capped by effective `nprobe`.

The existing default remains accumulated path+leaf scoring with a `1.0` overfetch multiplier.

## Code Under Review

- `src/am/ec_spire/options/mod.rs`: GUCs, scan-plan fields, route-budget resolver.
- `src/am/ec_spire/scan/routing.rs`: final leaf-ranking mode and selected-leaf cap.
- `src/am/ec_spire/scan/{candidates,snapshot}.rs` and coordinator snapshots: thread scan-plan ranking/budget metadata through existing route paths.
- Unit fixtures updated for the new scan-plan fields.

## Validation

- `artifacts/cargo-test-recursive-route-budget.log`
  - `cargo test -p ecaz recursive_route_budget --no-default-features --features pg18`
  - Result: `2 passed; 0 failed`.
- `artifacts/cargo-test-leaf-score-routing.log`
  - `cargo test -p ecaz route_recursive_routing_objects_to_leaf_routes_can_rank_final_leaves_by_leaf_score_only --no-default-features --features pg18`
  - Result: `1 passed; 0 failed`.

## Review Focus

1. Confirm the default path remains behavior-compatible: accumulated path+leaf ranking and `route_overfetch_multiplier = 1.0`.
2. Confirm leaf-score-only ranking is applied only at final leaf selection, not internal descent.
3. Confirm overfetch widens routing exploration without silently scanning more than effective `nprobe`.
4. Confirm this is an acceptable code-plumbing checkpoint before the Task 143 `ecaz bench suite` release A/B packets.
